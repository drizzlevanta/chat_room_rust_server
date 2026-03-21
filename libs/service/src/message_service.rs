use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::{constants::MAX_MESSAGE_LENGTH, message::Message, pagination::CursorPage};
use entity::message::Column as MessageColumn;
use entity::message::Entity as MessageEntity;
use entity::room::Column as RoomColumn;
use entity::user::Column as UserColumn;
use entity::user::Entity as UserEntity;
use sea_orm::QueryOrder;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    FromQueryResult, JoinType, QueryFilter, QuerySelect, RelationTrait, Select,
};
use thiserror::Error;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::cache::{ChatCache, IdempotencyKey, LATEST_MESSAGES_CACHE_LIMIT};
use crate::mappers::{EntityToDomain, message_mapper::MessageContext};

/// Service layer for message-related operations. This is where business logic related to messages is implemented, such as validation,
/// idempotency handling, and complex queries. The service interacts with the database through SeaORM.
pub struct MessageService {
    db: DatabaseConnection,
    cache: ChatCache,
}

impl MessageService {
    pub fn new(db: DatabaseConnection, cache: ChatCache) -> Self {
        Self { db, cache }
    }

    /// Base query that selects message fields with joined user/room public IDs,
    /// filtered by `room_id`. All message-listing functions build on top of this.
    fn base_message_query() -> Select<MessageEntity> {
        MessageEntity::find()
            .select_only()
            .column_as(MessageColumn::PublicId, "id")
            .column(MessageColumn::CreatedAt)
            .column(MessageColumn::Content)
            .column_as(UserColumn::PublicId, "sender")
            .column_as(RoomColumn::PublicId, "room")
            .join(JoinType::InnerJoin, entity::message::Relation::User.def())
            .join(JoinType::InnerJoin, entity::message::Relation::Room.def())
        // .filter(RoomColumn::PublicId.eq(room_id))
    }

    /// Add a message to a room.
    ///
    /// `idempotency_key` is a client-generated UUID that prevents duplicate
    /// messages on retries.  If the same key is sent again within the cache
    /// TTL (5 minutes), the previously created message is returned instead
    /// of inserting a new one.
    #[instrument(skip(self), name = "MessageService::add_message", err)]
    pub async fn add_message(
        &self,
        content: &str,
        user_id: Uuid,
        room_id: Uuid,
        idempotency_key: Uuid,
    ) -> Result<Message, MessageServiceError> {
        // Check idempotency cache first — return cached message on retry
        let cache_key = IdempotencyKey {
            user_id,
            key: idempotency_key,
        };

        self.cache
            .idempotency_message
            .try_get_with(cache_key, self.add_message_inner(content, user_id, room_id))
            .await
            .map_err(|e: Arc<MessageServiceError>| Arc::unwrap_or_clone(e))
    }

    #[instrument(skip(self), name = "MessageService::add_message_inner", err)]
    async fn add_message_inner(
        &self,
        content: &str,
        user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Message, MessageServiceError> {
        // Validate message length, counting by characters
        let char_count = content.chars().count();
        if char_count > MAX_MESSAGE_LENGTH {
            warn!(
                length = char_count,
                max = MAX_MESSAGE_LENGTH,
                "message too long"
            );
            return Err(MessageServiceError::MessageTooLong);
        }

        // Check if user exists in the room (single query with join)
        let user_in_room = UserEntity::find()
            .select_only()
            .column(UserColumn::Id)
            .column_as(RoomColumn::Id, "room_id")
            .join(JoinType::InnerJoin, entity::user::Relation::Room.def())
            .filter(UserColumn::PublicId.eq(user_id))
            .filter(RoomColumn::PublicId.eq(room_id))
            .into_tuple::<(i32, i32)>()
            .one(&self.db)
            .await? // Bubble up DB errors
            .ok_or(MessageServiceError::UserNotFoundInRoom { user_id, room_id })?; // Return error if no such user in room

        // Insert message
        let message = entity::message::ActiveModel {
            content: Set(content.to_string()),
            sender: Set(user_in_room.0),
            room: Set(user_in_room.1),
            ..Default::default()
        };

        let message = message.insert(&self.db).await?;
        debug!("inserted into DB");

        // Invalidate the cached first page for this room
        self.cache.invalidate_messages(&room_id).await;

        // Convert to domain message
        let domain_message = message.entity_to_domain(MessageContext { user_id, room_id });

        Ok(domain_message)
    }

    /// Fetch a single message by its public_id
    #[instrument(skip(self), name = "MessageService::get_message_by_id", err)]
    async fn get_message_by_id(&self, message_id: Uuid) -> Result<Message, MessageServiceError> {
        let row = Self::base_message_query()
            .filter(MessageColumn::PublicId.eq(message_id))
            .into_model::<MessageRow>()
            .one(&self.db)
            .await?
            .ok_or(MessageServiceError::MessageNotFound(message_id))?;

        Ok(Message::from(row))
    }

    /// Get all messages in a room. Use with caution for rooms with large message histories, as it loads everything into memory.

    #[instrument(skip(self), name = "MessageService::get_all_messages_in_room", err)]
    pub async fn get_all_messages_in_room(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<Message>, MessageServiceError> {
        let message_rows = Self::base_message_query()
            .filter(RoomColumn::PublicId.eq(room_id))
            .into_model::<MessageRow>()
            .all(&self.db)
            .await?;

        Ok(message_rows.into_iter().map(Message::from).collect())
    }

    /// Get messages in a room using cursor-based pagination.
    ///
    /// Messages are returned in reverse chronological order (newest first).
    /// The cursor is a UUIDv7 `public_id` — since UUIDv7 is time-ordered,
    /// sorting by it is equivalent to sorting by creation time, but without
    /// the duplicate-timestamp ambiguity that `created_at` cursors have.
    ///
    /// Pass `None` as `cursor` to start from the most recent messages.
    /// Use the returned `next_cursor` value to fetch the next page.
    #[instrument(skip(self), name = "MessageService::get_messages_in_room", err)]
    pub async fn get_messages_in_room(
        &self,
        room_id: Uuid,
        cursor: Option<Uuid>,
        limit: u64,
    ) -> Result<CursorPage<Message>, MessageServiceError> {
        // For small limits and first page, try the cache first
        if limit <= LATEST_MESSAGES_CACHE_LIMIT && cursor.is_none() {
            debug!("checking cache for latest messages");
            // Try cache first, if not found, query the database to populate the full page, then slice
            let cached_page = self
                .cache
                .latest_messages
                .try_get_with(
                    room_id,
                    self.get_messages_in_room_inner(room_id, None, LATEST_MESSAGES_CACHE_LIMIT),
                )
                .await
                .map_err(|e: Arc<MessageServiceError>| Arc::unwrap_or_clone(e))?;

            // Slice down to the requested limit
            let limit_usize = limit as usize;
            if limit_usize >= cached_page.items.len() {
                return Ok(cached_page);
            }
            let items = cached_page.items[..limit_usize].to_vec();
            let next_cursor = items.last().map(|m| m.id);
            return Ok(CursorPage {
                items,
                next_cursor,
                has_next_page: true, // always true since we sliced the cached page
            });
        } else {
            // For larger limits or cache misses, query the database directly
            debug!("cache miss or pagination beyond first page, querying DB");
            self.get_messages_in_room_inner(room_id, cursor, limit)
                .await
        }
    }

    #[instrument(skip(self), name = "MessageService::get_messages_in_room_inner", err)]
    async fn get_messages_in_room_inner(
        &self,
        room_id: Uuid,
        cursor: Option<Uuid>,
        limit: u64,
    ) -> Result<CursorPage<Message>, MessageServiceError> {
        let mut query = Self::base_message_query()
            .filter(RoomColumn::PublicId.eq(room_id))
            .order_by_desc(MessageColumn::PublicId);

        // Fetch messages with IDs less than the cursor for reverse chronological order
        // If cursor is None, this condition is ignored and we start from the most recent messages
        if let Some(cursor_id) = cursor {
            query = query.filter(MessageColumn::PublicId.lt(cursor_id));
        }

        // Fetch limit + 1 to detect if there's a next page
        let rows = query
            .limit(limit + 1)
            .into_model::<MessageRow>()
            .all(&self.db)
            .await?;

        let has_next_page = rows.len() as u64 > limit;

        // Take only the requested number of items for the current page
        let items: Vec<Message> = rows
            .into_iter()
            .take(limit as usize)
            .map(Message::from)
            .collect();

        // Use the last item's ID as the next cursor if there is a next page
        let next_cursor = if has_next_page {
            items.last().map(|m| m.id)
        } else {
            None
        };

        debug!(next_cursor = ?next_cursor, "fetched page");

        Ok(CursorPage {
            items,
            next_cursor,
            has_next_page,
        })
    }

    /// Delete a message by its public id
    #[instrument(skip(self), name = "MessageService::delete_message", err)]
    pub async fn delete_message(&self, message_id: Uuid) -> Result<Uuid, MessageServiceError> {
        // Single joined query: find the message and its room's public_id
        let (msg_id, room_public_id) = MessageEntity::find()
            .select_only()
            .column(MessageColumn::Id)
            .column_as(RoomColumn::PublicId, "room_public_id")
            .join(JoinType::InnerJoin, entity::message::Relation::Room.def())
            .filter(MessageColumn::PublicId.eq(message_id))
            .into_tuple::<(i32, Uuid)>()
            .one(&self.db)
            .await?
            .ok_or(MessageServiceError::MessageNotFound(message_id))?;

        // Delete by primary key — no second lookup needed
        MessageEntity::delete_by_id(msg_id).exec(&self.db).await?;

        info!("deleted message");

        // Invalidate the cached first page for this room
        self.cache.invalidate_messages(&room_public_id).await;

        Ok(message_id)
    }
}

/// Errors that can occur in MessageService operations.
#[derive(Error, Debug, Clone)]
pub enum MessageServiceError {
    #[error("Message too long. Maximum length is {MAX_MESSAGE_LENGTH} characters.")]
    MessageTooLong,

    #[error("User with id {user_id} not found in room with id {room_id}")]
    UserNotFoundInRoom { user_id: Uuid, room_id: Uuid },

    #[error("Room with id {0} not found")]
    RoomNotFound(Uuid),

    #[error("Message with id {0} not found")]
    MessageNotFound(Uuid),

    #[error("Database error in MessageService: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
}

// Struct to hold the result of the joined query
#[derive(FromQueryResult)]
struct MessageRow {
    id: Uuid,
    created_at: DateTime<Utc>,
    content: String,
    sender: Uuid,
    room: Uuid,
}

// Implement conversion from MessageRow to domain Message
impl From<MessageRow> for Message {
    fn from(row: MessageRow) -> Self {
        Message {
            id: row.id,
            created_at: row.created_at,
            content: row.content,
            sender: row.sender,
            room: row.room,
        }
    }
}

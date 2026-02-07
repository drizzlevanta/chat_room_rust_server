use chrono::{DateTime, Utc};
use domain::{constants::MAX_MESSAGE_LENGTH, message::Message};
use entity::message::Column as MessageColumn;
use entity::message::Entity as MessageEntity;
use entity::room::Column as RoomColumn;
use entity::room::Entity as RoomEntity;
use entity::user::Column as UserColumn;
use entity::user::Entity as UserEntity;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    FromQueryResult, JoinType, QueryFilter, QuerySelect, RelationTrait, prelude::DateTimeUtc,
};
use thiserror::Error;
use uuid::Uuid;

use crate::mappers::{EntityToDomain, message_mapper::MessageContext};

pub struct MessageService {
    pub db: DatabaseConnection,
}

impl MessageService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // TODO need to consider retry logic for the same message being sent multiple times
    pub async fn add_message(
        &self,
        content: &str,
        user_id: Uuid,
        room_id: Uuid,
    ) -> Result<Message, MessageServiceError> {
        // Validate message length
        if content.len() > MAX_MESSAGE_LENGTH {
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
            .await? // trailing ? to propagate database errors
            .ok_or(MessageServiceError::UserNotFoundinRoom { user_id, room_id })?; // `ok_or`convert the Option to Result, returning an error if None

        // Insert message
        let message = entity::message::ActiveModel {
            content: Set(content.to_string()),
            sender: Set(user_in_room.0),
            room: Set(user_in_room.1),
            ..Default::default()
        };

        let message = message.insert(&self.db).await?;

        // Return domain message
        let domain_message = message.entity_to_domain(MessageContext { user_id, room_id });

        Ok(domain_message)
    }

    pub async fn get_messages_in_room(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<Message>, MessageServiceError> {
        // Find the internal id for the room
        let internal_room_id = RoomEntity::find()
            .select_only()
            .column(RoomColumn::Id)
            .filter(RoomColumn::PublicId.eq(room_id))
            .into_tuple::<i32>()
            .one(&self.db)
            .await?
            .ok_or(MessageServiceError::RoomNotFound(room_id))?;

        //TODO optimize query to not fetch the entire user entity if not needed
        let message_and_users = MessageEntity::find()
            .filter(MessageColumn::Room.eq(internal_room_id))
            .find_also_related(UserEntity)
            .all(&self.db)
            .await?;
        // let message_and_users = MessageEntity::find()
        //     .filter(MessageColumn::Room.eq(internal_room_id))
        //     .find_also_related(UserEntity)
        //     .select_only_related()
        //     .all(&self.db)
        //     .await?;

        let domain_messages = message_and_users
            .into_iter()
            .filter_map(|(msg, user)| {
                user.map(|u| {
                    msg.entity_to_domain(MessageContext {
                        user_id: u.public_id,
                        room_id,
                    })
                }) // filter_map automatically filters out None values
            })
            .collect();

        Ok(domain_messages)
    }

    pub async fn get_all_messages_in_room(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<Message>, MessageServiceError> {
        // Select only necessary columns to avoid loading entire entities
        let message_rows = MessageEntity::find()
            .select_only()
            .column_as(MessageColumn::PublicId, "id")
            .column(MessageColumn::CreatedAt)
            .column(MessageColumn::Content)
            .column_as(UserColumn::PublicId, "sender")
            .column_as(RoomColumn::PublicId, "room")
            .join(JoinType::InnerJoin, entity::message::Relation::User.def())
            .join(JoinType::InnerJoin, entity::message::Relation::Room.def())
            .filter(RoomColumn::PublicId.eq(room_id))
            .into_model::<MessageRow>()
            .all(&self.db)
            .await?;

        // TODO externalize mapping if needed
        let domain_messages = message_rows
            .into_iter()
            .map(|row| Message {
                id: row.id,
                created_at: row.created_at,
                content: row.content,
                sender: row.sender,
                room: row.room,
            })
            .collect();

        Ok(domain_messages)
    }
}

#[derive(Error, Debug)]
pub enum MessageServiceError {
    #[error("Message too long. Maximum length is {MAX_MESSAGE_LENGTH} characters.")]
    MessageTooLong,

    #[error("User with id {user_id} not found in room with id {room_id}")]
    UserNotFoundinRoom { user_id: Uuid, room_id: Uuid },

    #[error("Room with id {0} not found")]
    RoomNotFound(Uuid),

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

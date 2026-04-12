use std::sync::Arc;

use chrono::Utc;
use domain::config::AppConfig;
use domain::events::{RoomEvent, TypingEvent};
use domain::room::Room as DomainRoom;
use entity::room::Column as RoomColumn;
use entity::room::Entity as RoomEntity;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use thiserror::Error;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::cache::{ChatCache, TypingIndicatorKey};
use crate::event_bus::EventBus;
use crate::mappers::EntityToDomain;

/// Service layer for room-related operations. This is where business logic related to rooms is implemented, such as validation,
/// status updates, and complex queries. The service interacts with the database through SeaORM.
pub struct RoomService {
    db: DatabaseConnection,
    cache: ChatCache,
    event_bus: EventBus,
    config: Arc<AppConfig>,
}

impl RoomService {
    pub fn new(
        db: DatabaseConnection,
        cache: ChatCache,
        event_bus: EventBus,
        config: Arc<AppConfig>,
    ) -> Self {
        Self {
            db,
            cache,
            event_bus,
            config,
        }
    }

    /// Add a new room with the given name and optional capacity.
    ///
    /// `idempotency_key` is a client-generated UUID that prevents duplicate
    /// rooms on retries. If the same key is sent again within the cache
    /// TTL, the previously created room is returned instead of inserting a new one.
    #[instrument(skip(self), name = "RoomService::add_room", err, ret(level = "debug"))]
    pub async fn add_room(
        &self,
        name: String,
        capacity: Option<u32>,
        idempotency_key: Uuid,
    ) -> Result<DomainRoom, RoomServiceError> {
        self.cache
            .idempotency_room
            .try_get_with(idempotency_key, self.add_room_inner(name, capacity))
            .await
            .map_err(|e: Arc<RoomServiceError>| Arc::unwrap_or_clone(e))
    }

    #[instrument(
        skip(self),
        name = "RoomService::add_room_inner",
        err,
        ret(level = "debug")
    )]
    async fn add_room_inner(
        &self,
        name: String,
        capacity: Option<u32>,
    ) -> Result<DomainRoom, RoomServiceError> {
        // Validate capacity. This should be done at the service layer since we might have multiple entry points (e.g. REST API, GraphQL, CLI) that can create rooms.
        if let Some(cap) = capacity {
            if cap > self.config.room.max_capacity {
                warn!(cap, max = self.config.room.max_capacity, "capacity exceeded");
                return Err(RoomServiceError::MaxCapacityExceeded(cap));
            }
        }

        let room = entity::room::ActiveModel {
            name: Set(name),
            capacity: Set(capacity.unwrap_or(self.config.room.default_capacity) as i32),
            description: Set(None),
            ..Default::default()
        };

        let room = room.insert(&self.db).await?;
        debug!(room_id = %room.public_id, "inserted into DB");

        // Invalidate the cache for all rooms since we added a new one
        self.cache.invalidate_all_rooms().await;

        // Add to cache before returning so that subsequent reads can hit the cache
        let room = room.entity_to_domain(());
        self.cache.rooms.insert(room.id, room.clone()).await;
        debug!(room_id = %room.id, "cached room");

        // Publish to event bus so subscribers can react to the new room
        self.event_bus.room.publish(RoomEvent::Added(room.clone()));

        Ok(room)
    }

    /// Fetch a room by its public ID.
    #[instrument(skip(self), name = "RoomService::get_room_by_id", err)]
    pub async fn get_room_by_id(&self, room_id: Uuid) -> Result<DomainRoom, RoomServiceError> {
        let room = self
            .cache
            .rooms
            .try_get_with(room_id, async {
                debug!("cache miss, querying DB");
                let room = RoomEntity::find()
                    .filter(RoomColumn::PublicId.eq(room_id))
                    .one(&self.db)
                    .await?
                    .ok_or(RoomServiceError::RoomNotFound(room_id))?;

                Ok(room.entity_to_domain(())) as Result<DomainRoom, RoomServiceError>
            })
            .await
            .map_err(|e: Arc<RoomServiceError>| Arc::unwrap_or_clone(e))?;

        Ok(room)
    }

    /// Fetch all rooms.
    #[instrument(skip(self), name = "RoomService::get_all_rooms", err)]
    pub async fn get_all_rooms(&self) -> Result<Vec<DomainRoom>, RoomServiceError> {
        let rooms = self
            .cache
            .all_rooms
            .try_get_with((), async {
                debug!("cache miss, querying DB");
                let rooms = RoomEntity::find().all(&self.db).await?;

                let domain_rooms: Vec<DomainRoom> = rooms
                    .into_iter()
                    .map(|room| room.entity_to_domain(()))
                    .collect();

                info!(count = domain_rooms.len(), "fetched all rooms");
                Ok(domain_rooms) as Result<Vec<DomainRoom>, RoomServiceError>
            })
            .await
            .map_err(|e: Arc<RoomServiceError>| Arc::unwrap_or_clone(e))?;

        Ok(rooms)
    }

    /// Update a room's name, description, or capacity.
    ///
    /// Only fields set to `Some` are updated; `None` fields are left unchanged.
    #[instrument(skip(self), name = "RoomService::update_room", err)]
    pub async fn update_room(
        &self,
        room_id: Uuid,
        name: Option<String>,
        description: Option<Option<String>>,
        capacity: Option<u32>,
    ) -> Result<DomainRoom, RoomServiceError> {
        if name.is_none() && description.is_none() && capacity.is_none() {
            debug!("no fields to update, returning current state");
            return self.get_room_by_id(room_id).await;
        }

        if let Some(cap) = capacity {
            if cap > self.config.room.max_capacity {
                warn!(cap, max = self.config.room.max_capacity, "capacity exceeded");
                return Err(RoomServiceError::MaxCapacityExceeded(cap));
            }
        }

        let room = RoomEntity::find()
            .filter(RoomColumn::PublicId.eq(room_id))
            .one(&self.db)
            .await?
            .ok_or(RoomServiceError::RoomNotFound(room_id))?;

        let mut active: entity::room::ActiveModel = room.into();

        if let Some(name) = name {
            active.name = Set(name);
        }

        if let Some(description) = description {
            active.description = Set(description);
        }

        if let Some(capacity) = capacity {
            active.capacity = Set(capacity as i32);
        }

        let updated = active.update(&self.db).await?;
        debug!("updated in DB");

        self.cache.invalidate_room(&room_id).await;

        let room = updated.entity_to_domain(());
        self.cache.rooms.insert(room.id, room.clone()).await;
        debug!("cache refreshed");

        Ok(room)
    }

    #[instrument(skip(self), name = "RoomService::delete_room", err)]
    pub async fn delete_room(&self, room_id: Uuid) -> Result<Uuid, RoomServiceError> {
        let delete_result = RoomEntity::delete_many()
            .filter(RoomColumn::PublicId.eq(room_id))
            .exec(&self.db)
            .await?;

        if delete_result.rows_affected == 0 {
            warn!("room not found for deletion");
            return Err(RoomServiceError::RoomNotFound(room_id));
        }

        self.cache.invalidate_room(&room_id).await;

        // Publish to event bus so subscribers can react to the deleted room
        self.event_bus.room.publish(RoomEvent::Removed(room_id));
        info!("room deleted");

        Ok(room_id)
    }

    /// Set a user's typing status in a room.
    ///
    /// This is ephemeral state — not persisted to the database — published
    /// only through the event bus for real-time subscriptions.
    ///
    /// **Debounce strategy:**
    /// - `is_typing = true`: only published when no recent event exists in the
    ///   cache (TTL-based debounce). Repeated "start typing" calls within the
    ///   TTL window are silently dropped to avoid flooding subscribers.
    /// - `is_typing = false`: always published immediately and the debounce
    ///   guard is removed, so the next "start typing" will go through.
    #[instrument(skip(self), name = "RoomService::set_user_typing", err)]
    pub async fn set_user_typing(
        &self,
        room_id: Uuid,
        user_id: Uuid,
        is_typing: bool,
    ) -> Result<(), RoomServiceError> {
        let key = TypingIndicatorKey { room_id, user_id };

        if is_typing {
            // Debounce: if entry exists in cache, we published recently. Skip publishing.
            if self.cache.typing_indicators.get(&key).await.is_some() {
                debug!("typing status debounced, skipping update");
                return Ok(());
            }
            self.cache.typing_indicators.insert(key, ()).await;
        } else {
            // Always publish stop immediately, remove debounce guard from cache
            self.cache.typing_indicators.invalidate(&key).await;
        }

        // Publish to event bus
        self.event_bus
            .room
            .publish(RoomEvent::UserTyping(TypingEvent {
                user_id,
                room_id,
                is_typing,
                timestamp: Utc::now(),
            }));
        debug!("typing status updated");

        Ok(())
    }
}

#[derive(Error, Debug, Clone)]
pub enum RoomServiceError {
    #[error("Room not found with id: {0}")]
    RoomNotFound(Uuid),

    #[error("Failed to add room: {0}")]
    RoomNotAdded(String),

    #[error("Max Room capacity exceeded: {0}")]
    MaxCapacityExceeded(u32),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
}

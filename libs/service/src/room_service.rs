use std::sync::Arc;

use domain::constants::{DEFAULT_ROOM_CAPACITY, MAX_ROOM_CAPACITY};
use domain::room::Room as DomainRoom;
use entity::room::Column as RoomColumn;
use entity::room::Entity as RoomEntity;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use thiserror::Error;
use uuid::Uuid;

use crate::cache::ChatCache;
use crate::mappers::EntityToDomain;

/// Service layer for room-related operations. This is where business logic related to rooms is implemented, such as validation,
/// status updates, and complex queries. The service interacts with the database through SeaORM.
pub struct RoomService {
    db: DatabaseConnection,
    cache: ChatCache,
}

impl RoomService {
    pub fn new(db: DatabaseConnection, cache: ChatCache) -> Self {
        Self { db, cache }
    }

    /// Add a new room with the given name and optional capacity.
    pub async fn add_room(
        &self,
        name: String,
        capacity: Option<u32>,
    ) -> Result<DomainRoom, RoomServiceError> {
        // Validate capacity. This should be done at the service layer since we might have multiple entry points (e.g. REST API, GraphQL, CLI) that can create rooms.
        if let Some(cap) = capacity {
            if cap > MAX_ROOM_CAPACITY {
                return Err(RoomServiceError::MaxCapacityExceeded(cap));
            }
        }

        let room = entity::room::ActiveModel {
            name: Set(name),
            capacity: Set(capacity
                .map(|c| i32::try_from(c).unwrap_or(DEFAULT_ROOM_CAPACITY))
                .unwrap_or(DEFAULT_ROOM_CAPACITY)),
            description: Set(None),
            ..Default::default()
        };

        let room = room.insert(&self.db).await?;

        // Invalidate the cache for all rooms since we added a new one
        self.cache.invalidate_all_rooms().await;

        // Add to cache before returning so that subsequent reads can hit the cache
        let room = room.entity_to_domain(());
        self.cache.rooms.insert(room.id, room.clone()).await;

        Ok(room)
    }

    /// Fetch a room by its public ID.
    pub async fn get_room_by_id(&self, room_id: Uuid) -> Result<DomainRoom, RoomServiceError> {
        // Check cache first, if not found, fetch from DB and populate cache before returning
        let room = self
            .cache
            .rooms
            .try_get_with(room_id, async {
                let room = RoomEntity::find()
                    .filter(RoomColumn::PublicId.eq(room_id))
                    .one(&self.db)
                    .await? // bubble up DB errors
                    .ok_or(RoomServiceError::RoomNotFound(room_id))?; // convert not found to service error

                Ok(room.entity_to_domain(())) as Result<DomainRoom, RoomServiceError>
            })
            .await
            .map_err(|e: Arc<RoomServiceError>| Arc::unwrap_or_clone(e))?; // unwrap Arc to get the error

        Ok(room)
    }

    /// Fetch all rooms.
    pub async fn get_all_rooms(&self) -> Result<Vec<DomainRoom>, RoomServiceError> {
        // Check cache first, if not found, fetch from DB and populate cache before returning
        let rooms = self
            .cache
            .all_rooms
            .try_get_with((), async {
                let rooms = RoomEntity::find().all(&self.db).await?; // bubble up DB errors

                let domain_rooms: Vec<DomainRoom> = rooms
                    .into_iter()
                    .map(|room| room.entity_to_domain(()))
                    .collect();

                Ok(domain_rooms) as Result<Vec<DomainRoom>, RoomServiceError>
            })
            .await
            .map_err(|e: Arc<RoomServiceError>| Arc::unwrap_or_clone(e))?; // unwrap Arc to get the error

        Ok(rooms)
    }

    /// Update a room's name, description, or capacity.
    ///
    /// Only fields set to `Some` are updated; `None` fields are left unchanged.
    pub async fn update_room(
        &self,
        room_id: Uuid,
        name: Option<String>,
        description: Option<Option<String>>,
        capacity: Option<u32>,
    ) -> Result<DomainRoom, RoomServiceError> {
        // Nothing to update — return current state without writing
        if name.is_none() && description.is_none() && capacity.is_none() {
            return self.get_room_by_id(room_id).await;
        }

        // Validate capacity if provided
        if let Some(cap) = capacity {
            if cap > MAX_ROOM_CAPACITY {
                return Err(RoomServiceError::MaxCapacityExceeded(cap));
            }
        }

        // Fetch the existing room
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
            active.capacity = Set(i32::try_from(capacity).unwrap_or(DEFAULT_ROOM_CAPACITY));
        }

        // `updated_at` will be automatically set by the ActiveModelBehavior implementation
        let updated = active.update(&self.db).await?;

        // Invalidate cache for this room and all rooms list since we updated a room
        self.cache.invalidate_room(&room_id).await;

        // Re-populate individual room cache so subsequent reads hit cache
        let room = updated.entity_to_domain(());
        self.cache.rooms.insert(room.id, room.clone()).await;

        Ok(room)
    }

    pub async fn delete_room(&self, room_id: Uuid) -> Result<Uuid, RoomServiceError> {
        // Delete directly with a single query
        let delete_result = RoomEntity::delete_many()
            .filter(RoomColumn::PublicId.eq(room_id))
            .exec(&self.db)
            .await?;

        // Check if any rows were deleted
        if delete_result.rows_affected == 0 {
            return Err(RoomServiceError::RoomNotFound(room_id));
        }

        // Invalidate cache for this room and all rooms list since we deleted a room
        self.cache.invalidate_room(&room_id).await;

        Ok(room_id)
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

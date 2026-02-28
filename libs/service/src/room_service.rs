use domain::constants::{DEFAULT_ROOM_CAPACITY, MAX_ROOM_CAPACITY};
use domain::room::Room as DomainRoom;
use entity::room::Column as RoomColumn;
use entity::room::Entity as RoomEntity;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use thiserror::Error;
use uuid::Uuid;

use crate::mappers::EntityToDomain;

/// Service layer for room-related operations. This is where business logic related to rooms is implemented, such as validation,
/// status updates, and complex queries. The service interacts with the database through SeaORM.
pub struct RoomService {
    db: DatabaseConnection,
    // TODO add cache for rooms (e.g. Redis) to reduce DB load, especially for frequently accessed rooms and lists of rooms. Cache invalidation can be handled on room updates/deletions.
}

impl RoomService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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
            capacity: Set(capacity.map(|c| c as i32).unwrap_or(DEFAULT_ROOM_CAPACITY)),
            description: Set(None),
            ..Default::default()
        };

        let room = room.insert(&self.db).await?;
        Ok(room.entity_to_domain(()))
    }

    /// Fetch a room by its public ID.
    pub async fn get_room_by_id(&self, room_id: Uuid) -> Result<DomainRoom, RoomServiceError> {
        // Ok to select the entire row since it's just one room
        let room = RoomEntity::find()
            .filter(RoomColumn::PublicId.eq(room_id))
            .one(&self.db)
            .await?
            .ok_or(RoomServiceError::RoomNotFound(room_id))?;

        Ok(room.entity_to_domain(()))
    }

    /// Fetch all rooms.
    pub async fn get_all_rooms(&self) -> Result<Vec<DomainRoom>, RoomServiceError> {
        let rooms = RoomEntity::find().all(&self.db).await?;

        let domain_rooms = rooms
            .into_iter()
            .map(|room| room.entity_to_domain(()))
            .collect();

        Ok(domain_rooms)
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

        // Ok to unwrap since we checked for all None at the beginning
        active.name = Set(name.unwrap());
        active.description = Set(description.unwrap());
        active.capacity = Set(capacity.unwrap() as i32);

        // `updated_at` will be automatically set by the ActiveModelBehavior implementation
        let updated = active.update(&self.db).await?;
        Ok(updated.entity_to_domain(()))
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

        Ok(room_id)
    }
}

#[derive(Error, Debug)]
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

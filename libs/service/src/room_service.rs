use domain::room::Room as DomainRoom;
use entity::room::Entity as RoomEntity;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use thiserror::Error;
use uuid::Uuid;

use crate::mappers::EntityToDomain;

pub struct RoomService {
    db: DatabaseConnection,
}

impl RoomService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn add_room(
        &self,
        name: String,
        capacity: Option<u32>,
    ) -> Result<DomainRoom, RoomServiceError> {
        // Validate capacity
        if let Some(cap) = capacity {
            if cap > 1000 {
                //TODO add config for max capacity
                return Err(RoomServiceError::MaxCapacityExceeded(cap));
            }
        }

        let room = entity::room::ActiveModel {
            name: Set(name),
            capacity: Set(capacity.map(|c| c as i32).unwrap_or(100)), // Default capacity
            description: Set(None),
            public_id: Set(Uuid::new_v4()),
            ..Default::default()
        };

        let room = room.insert(&self.db).await?;
        Ok(room.entity_to_domain(()))
    }

    pub async fn get_room_by_id(&self, room_id: Uuid) -> Result<DomainRoom, RoomServiceError> {
        let room = RoomEntity::find()
            .filter(entity::room::Column::PublicId.eq(room_id))
            .one(&self.db)
            .await?
            .ok_or(RoomServiceError::RoomNotFound(room_id))?;

        Ok(room.entity_to_domain(()))
    }

    pub async fn get_all_rooms(&self) -> Result<Vec<DomainRoom>, RoomServiceError> {
        let rooms = RoomEntity::find().all(&self.db).await?;

        let domain_rooms = rooms
            .into_iter()
            .map(|room| room.entity_to_domain(()))
            .collect();

        Ok(domain_rooms)
    }

    pub async fn delete_room(&self, room_id: Uuid) -> Result<Uuid, RoomServiceError> {
        // Delete directly with a single query
        let delete_result = RoomEntity::delete_many()
            .filter(entity::room::Column::PublicId.eq(room_id))
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

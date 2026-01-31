use domain::user::{ParseUserStatusError, Status, User as DomainUser};
use entity::room::Entity as RoomEntity;
use entity::user::Entity as UserEntity;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use thiserror::Error;
use uuid::Uuid;

use crate::mappers::user_mapper;

pub struct UserService {
    db: DatabaseConnection, // In SeaORM, DatabaseConnection is internally an arc to a connection pool, therefore cheap to clone
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Adds a new user to the system.
    /// If a room id is provided, associates the user with that room.
    pub async fn add_user(
        &self,
        name: String,
        room: Option<Uuid>,
    ) -> Result<DomainUser, UserServiceError> {
        // Try to find the room entity if room id is provided
        let room_entity = if let Some(room_id) = room {
            RoomEntity::find()
                .filter(entity::room::Column::PublicId.eq(room_id))
                .one(&self.db)
                .await
                .map_err(|_| UserServiceError::RoomNotFound(room_id))?
        } else {
            None
        };

        // Try inserting user into the database
        let user = entity::user::ActiveModel {
            name: ActiveValue::Set(name.clone()),
            status: ActiveValue::Set(Some(Status::Online.to_string())),
            room: ActiveValue::Set(room_entity.as_ref().map(|r| r.id)), // Use internal Id
            last_seen_at: ActiveValue::Set(None),
            ..Default::default()
        };

        let user = user
            .insert(&self.db)
            .await
            .map_err(|_| UserServiceError::UserNotAdded(name.clone()))?;

        // Convert to domain user
        let user = user_mapper::entity_to_domain(user, room_entity)?;
        Ok(user)
    }

    /// Retrieves a user by their public Id.
    /// Uses a single database query with LEFT JOIN to fetch user and room together.
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<DomainUser, UserServiceError> {
        // Fetch user and room in a single query
        let (user_entity, room_entity) = UserEntity::find()
            .filter(entity::user::Column::PublicId.eq(user_id))
            .find_also_related(RoomEntity)
            .one(&self.db)
            .await?
            .ok_or(UserServiceError::UserNotFound(user_id))?;

        // Convert to domain user
        let user = user_mapper::entity_to_domain(user_entity, room_entity)?;
        Ok(user)
    }

    // Retrieves all users in a specific room by the room's public Id.
    pub async fn get_users_in_room(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<DomainUser>, UserServiceError> {
        let users = UserEntity::find()
            .filter(entity::user::Column::Room.eq(room_id))
            .all(&self.db)
            .await?;

        let domain_users = users
            .into_iter()
            .filter_map(|user_entity| user_mapper::entity_to_domain(user_entity, None).ok())
            .collect::<Vec<DomainUser>>();

        Ok(domain_users)
    }

    // Updates the status of a user
    pub async fn update_user_status(
        &self,
        user_id: Uuid,
        new_status: Status,
    ) -> Result<(), UserServiceError> {
        // Find the user by public id
        let user_entity = UserEntity::find()
            .filter(entity::user::Column::PublicId.eq(user_id))
            .one(&self.db)
            .await?
            .ok_or(UserServiceError::UserNotFound(user_id))?;

        // Update the user's status
        let mut user_active_model: entity::user::ActiveModel = user_entity.into();
        user_active_model.status = ActiveValue::Set(Some(new_status.to_string())); //TODO map status properly

        user_active_model
            .update(&self.db)
            .await
            .map_err(|e| UserServiceError::UserStatusUpdateFailed(e.to_string()))?;

        //TODO logging

        Ok(())
    }

    /// Deletes a user by their public Id.
    /// Uses a single database query to delete directly.
    pub async fn delete_user(&self, user_id: Uuid) -> Result<Uuid, UserServiceError> {
        // Delete directly with a single query
        let delete_result = UserEntity::delete_many()
            .filter(entity::user::Column::PublicId.eq(user_id))
            .exec(&self.db)
            .await?;

        // Check if any rows were deleted
        if delete_result.rows_affected == 0 {
            return Err(UserServiceError::UserNotFound(user_id));
        }

        //TODO logging

        Ok(user_id)
    }

    //  TODO get list of users by status
}

#[derive(Error, Debug)]
pub enum UserServiceError {
    #[error("Failed to add user: {0}")]
    UserNotAdded(String),

    #[error("User not found with id: {0}")]
    UserNotFound(Uuid),

    #[error("Room not found with id: {0}")]
    RoomNotFound(Uuid),

    #[error("Failed to update user status to: {0}")]
    UserStatusUpdateFailed(String),

    #[error("User already exists with name: {0}")]
    UserAlreadyExists(String),

    #[error(transparent)]
    InvalidStatus(#[from] ParseUserStatusError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
}

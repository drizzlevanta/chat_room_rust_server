use chrono::{DateTime, Utc};
use domain::user::{ParseUserStatusError, Status, User as DomainUser};
use entity::room::Column as RoomColumn;
use entity::room::Entity as RoomEntity;
use entity::user::Column as UserColumn;
use entity::user::Entity as UserEntity;
use sea_orm::ActiveValue;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType,
    QueryFilter, QuerySelect, RelationTrait, Select,
};
use thiserror::Error;
use uuid::Uuid;

use crate::mappers::{TryEntityToDomain, user_mapper::RoomParam};

/// Service layer for user-related operations. This is where business logic related to users is implemented, such as validation,
/// status updates, and complex queries. The service interacts with the database through SeaORM.
pub struct UserService {
    db: DatabaseConnection, // In SeaORM, DatabaseConnection is internally an arc to a connection pool, therefore cheap to clone
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Adds a new user to the system.
    /// If a room id is provided, associates the user with that room.
    /// If the room is not found, user is still created but without a room association.
    pub async fn add_user(
        &self,
        name: String,
        room: Option<Uuid>,
    ) -> Result<DomainUser, UserServiceError> {
        // Try to find the room entity if room id is provided
        // Okay to return the entire room entity since it's only one row
        let room_entity = if let Some(room_id) = room {
            RoomEntity::find()
                .filter(entity::room::Column::PublicId.eq(room_id))
                .one(&self.db)
                .await
                .unwrap_or(None) // If DB error occurs, treat as room not found
        } else {
            None
        };

        // Try inserting user
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
        let user = user.try_entity_to_domain(room_entity.map(RoomParam::Entity))?;
        Ok(user)
    }

    /// Base query that projects user columns (with room public ID via JOIN) into a `UserRow`.
    /// Callers can add filters before executing with `.one()` or `.all()`.
    fn user_row_query() -> Select<UserEntity> {
        UserEntity::find()
            .select_only()
            .join(JoinType::LeftJoin, entity::user::Relation::Room.def())
            .column_as(UserColumn::PublicId, "id")
            .column_as(UserColumn::Name, "name")
            .column_as(UserColumn::Status, "status")
            .column_as(RoomColumn::PublicId, "room")
            .column_as(UserColumn::LastSeenAt, "last_seen")
    }

    /// Retrieves a user by their public Id.
    /// Uses a single database query with JOIN to fetch user and room together.
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<DomainUser, UserServiceError> {
        let user_row = Self::user_row_query()
            .filter(UserColumn::PublicId.eq(user_id))
            .into_model::<UserRow>()
            .one(&self.db)
            .await?;

        Ok(user_row
            .ok_or(UserServiceError::UserNotFound(user_id))?
            .into())
    }

    /// Fetch users in a room by the room's public ID.
    pub async fn get_users_in_room(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<DomainUser>, UserServiceError> {
        let user_rows = Self::user_row_query()
            .filter(RoomColumn::PublicId.eq(room_id))
            .into_model::<UserRow>()
            .all(&self.db)
            .await?;

        let domain_users = user_rows.into_iter().map(DomainUser::from).collect();
        Ok(domain_users)
    }

    /// Updates the status of a user.
    /// Also sets `last_seen_at` to the current time when transitioning to `Offline` or `Away`.
    pub async fn update_user_status(
        &self,
        user_id: Uuid,
        new_status: Status,
    ) -> Result<(), UserServiceError> {
        // Update user status
        let mut query = UserEntity::update_many()
            .col_expr(UserColumn::Status, Expr::value(new_status.to_string()))
            .filter(UserColumn::PublicId.eq(user_id));

        // Record last_seen_at when user goes offline or away
        if matches!(new_status, Status::Offline | Status::Away) {
            query = query.col_expr(UserColumn::LastSeenAt, Expr::value(Utc::now()));
        }

        let result = query.exec(&self.db).await?;

        if result.rows_affected == 0 {
            return Err(UserServiceError::UserNotFound(user_id));
        }

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

        Ok(user_id)
    }

    /// Get list of users by status, optionally filtered by room.
    /// If room_id is provided, only users in that room with the given status are returned.
    pub async fn get_users_by_status(
        &self,
        status: Status,
        room_id: Option<Uuid>,
    ) -> Result<Vec<DomainUser>, UserServiceError> {
        // Get status string
        let status = status.to_string();

        let mut query = Self::user_row_query().filter(UserColumn::Status.eq(status));

        // If room_id is provided, filter by room as well
        if let Some(room_id) = room_id {
            query = query.filter(RoomColumn::PublicId.eq(room_id));
        }

        let user_rows = query.into_model::<UserRow>().all(&self.db).await?;

        let domain_users = user_rows.into_iter().map(DomainUser::from).collect();
        Ok(domain_users)
    }
}

/// Errors that can occur in UserService operations.
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

// Struct to hold the result of the User query
#[derive(FromQueryResult)]
struct UserRow {
    id: Uuid,
    name: String,
    status: Option<String>,
    room: Option<Uuid>,
    last_seen: Option<DateTime<Utc>>,
}

// Convert a UserRow query struct into a DomainUser.
impl From<UserRow> for DomainUser {
    fn from(row: UserRow) -> Self {
        DomainUser {
            id: row.id,
            name: row.name,
            status: row.status.and_then(|s| s.parse::<Status>().ok()),
            room: row.room,
            last_seen: row.last_seen,
        }
    }
}

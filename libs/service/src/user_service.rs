use std::sync::Arc;

use crate::cache::ChatCache;
use crate::event_bus::EventBus;
use chrono::{DateTime, Utc};
use domain::events::{RoomEvent, UserEvent};
use domain::user::{ParseUserStatusError, Status, User as DomainUser};
use entity::room::Column as RoomColumn;
use entity::room::Entity as RoomEntity;
use entity::user::Column as UserColumn;
use entity::user::Entity as UserEntity;
use entity::user::Relation as UserRelation;
use sea_orm::sea_query::{Expr, Query, SubQueryStatement};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult,
    JoinType, QueryFilter, QuerySelect, RelationTrait, Select,
};
use sea_orm::{ActiveValue, PaginatorTrait};
use thiserror::Error;
use tracing::error;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::mappers::{TryEntityToDomain, user_mapper::RoomParam};

/// Service layer for user-related operations. This is where business logic related to users is implemented, such as validation,
/// status updates, and complex queries. The service interacts with the database through SeaORM.
pub struct UserService {
    db: DatabaseConnection, // In SeaORM, DatabaseConnection is internally an arc to a connection pool, therefore cheap to clone
    cache: ChatCache,
    event_bus: EventBus,
}

impl UserService {
    pub fn new(db: DatabaseConnection, cache: ChatCache, event_bus: EventBus) -> Self {
        Self {
            db,
            cache,
            event_bus,
        }
    }

    /// Adds a new user to the system.
    /// If a room id is provided, associates the user with that room.
    /// If the room is not found, user is still created but without a room association.
    #[instrument(skip(self), name = "UserService::add_user", err)]
    pub async fn add_user(
        &self,
        name: String,
        room: Option<Uuid>,
        idempotency_key: Uuid,
    ) -> Result<DomainUser, UserServiceError> {
        // Use idempotency cache to ensure that retries with the same idempotency key return the same result without creating duplicate users.
        self.cache
            .idempotency_user
            .try_get_with(
                idempotency_key,
                self.add_user_inner(name, room), // The inner function contains the actual logic to add a user, while the outer function handles idempotency caching.
            )
            .await
            .map_err(|e| Arc::unwrap_or_clone(e))
    }

    #[instrument(skip(self), name = "UserService::add_user_inner", err)]
    async fn add_user_inner(
        &self,
        name: String,
        room: Option<Uuid>,
    ) -> Result<DomainUser, UserServiceError> {
        // Try to find the room entity if room id is provided
        // Okay to return the entire room entity since it's only one row
        let room_entity = if let Some(room_id) = room {
            RoomEntity::find()
                .filter(RoomColumn::PublicId.eq(room_id))
                .one(&self.db)
                .await
                .unwrap_or_else(|e| {
                    warn!(error = %e, "room lookup failed, creating user without room");
                    None
                }) // If DB error occurs, treat as room not found to allow user creation to proceed without room association
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

        let user = user.insert(&self.db).await.map_err(|e| {
            if matches!(
                e.sql_err(),
                Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
            ) {
                UserServiceError::UserAlreadyExists(name.clone())
            } else {
                UserServiceError::UserNotAdded(name.clone())
            }
        })?;

        debug!("inserted into DB");

        // Convert to domain user
        let user = user.try_entity_to_domain(room_entity.map(RoomParam::Entity))?;

        self.cache.users.insert(user.id, user.clone()).await;
        if let Some(room_id) = room {
            self.cache.users_in_room.invalidate(&room_id).await;
            // Publish user joined event to event bus if the user was associated with a room
            self.event_bus.room.publish(RoomEvent::UserEntered {
                room_id,
                user_id: user.id,
            });
        }
        debug!("user cached");

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
    #[instrument(skip(self), name = "UserService::get_user_by_id", err)]
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<DomainUser, UserServiceError> {
        // Check cache first
        let user = self
            .cache
            .users
            .try_get_with(user_id, async {
                debug!("cache miss, querying DB");
                let user_row = Self::user_row_query()
                    .filter(UserColumn::PublicId.eq(user_id))
                    .into_model::<UserRow>()
                    .one(&self.db)
                    .await?;

                let user = user_row
                    .ok_or(UserServiceError::UserNotFound(user_id))?
                    .into();

                Ok(user) as Result<DomainUser, UserServiceError>
            })
            .await
            .map_err(|e| Arc::unwrap_or_clone(e))?;

        Ok(user)
    }

    /// Fetch users in a room by the room's public ID.
    #[instrument(skip(self), name = "UserService::get_users_in_room", err)]
    pub async fn get_users_in_room(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<DomainUser>, UserServiceError> {
        // Check cache first
        let users = self
            .cache
            .users_in_room
            .try_get_with(room_id, async {
                debug!("cache miss, querying DB");
                let user_rows = Self::user_row_query()
                    .filter(RoomColumn::PublicId.eq(room_id))
                    .into_model::<UserRow>()
                    .all(&self.db)
                    .await?;

                let domain_users = user_rows.into_iter().map(DomainUser::from).collect();
                Ok(domain_users) as Result<Vec<DomainUser>, UserServiceError>
            })
            .await
            .map_err(|e| Arc::unwrap_or_clone(e))?;

        Ok(users)
    }

    /// Updates the status of a user.
    /// Also sets `last_seen_at` to the current time when transitioning to `Offline` or `Away`.
    #[instrument(skip(self), name = "UserService::update_user_status", err)]
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
            warn!("user not found for status update");
            return Err(UserServiceError::UserNotFound(user_id));
        }

        self.cache.users.invalidate(&user_id).await;

        self.event_bus.user.publish(UserEvent::StatusChanged {
            user_id,
            status: new_status,
        });

        info!("user status updated");

        Ok(())
    }

    /// Deletes a user by their public Id.
    /// Uses a single database query to delete directly.
    #[instrument(skip(self), name = "UserService::delete_user", err)]
    pub async fn delete_user(&self, user_id: Uuid) -> Result<Uuid, UserServiceError> {
        // Delete directly with a single query
        let delete_result = UserEntity::delete_many()
            .filter(UserColumn::PublicId.eq(user_id))
            .exec(&self.db)
            .await?;

        // Check if any rows were deleted
        if delete_result.rows_affected == 0 {
            warn!("user not found for deletion");
            return Err(UserServiceError::UserNotFound(user_id));
        }

        self.cache.users.invalidate(&user_id).await;
        info!("user deleted");

        Ok(user_id)
    }

    /// Get list of users by status, optionally filtered by room.
    /// If room_id is provided, only users in that room with the given status are returned.
    #[instrument(skip(self), name = "UserService::get_users_by_status", err)]
    pub async fn get_users_by_status(
        &self,
        status: Status,
        room_id: Option<Uuid>,
    ) -> Result<Vec<DomainUser>, UserServiceError> {
        // Bypass the cache for this query since it's more dynamic and less likely to be repeated frequently, and caching would be more complex due to the combination of filters (status + optional room).
        // Get status string
        let status = status.to_string();

        let mut query = Self::user_row_query().filter(UserColumn::Status.eq(status));

        // If room_id is provided, filter by room as well
        if let Some(room_id) = room_id {
            query = query.filter(RoomColumn::PublicId.eq(room_id));
        }

        let user_rows = query.into_model::<UserRow>().all(&self.db).await?;
        debug!("queried DB for users by status");

        let domain_users = user_rows.into_iter().map(DomainUser::from).collect();
        Ok(domain_users)
    }

    /// User leaves their current room (if any). Sets user's room to null
    #[instrument(skip(self), name = "UserService::leave_room", err)]
    pub async fn leave_room(&self, user_id: Uuid, room_id: Uuid) -> Result<(), UserServiceError> {
        // Set user's room to null
        let result = UserEntity::update_many()
            .col_expr(UserColumn::Room, Expr::value(None as Option<Uuid>))
            .filter(UserColumn::PublicId.eq(user_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            warn!("user not found for leave_room");
            return Err(UserServiceError::UserNotFound(user_id));
        }

        // Invalidate the user's cache entry since their room association has changed
        self.cache.users.invalidate(&user_id).await;

        // Invalidate users_in_room cache for the room they left
        self.cache.users_in_room.invalidate(&room_id).await;

        // Publish user left event to event bus
        self.event_bus
            .room
            .publish(RoomEvent::UserLeft { room_id, user_id });
        Ok(())
    }

    /// User joins a room. Sets user's room to the specified room id.
    /// If the room does not exist, returns an error.
    #[instrument(skip(self), name = "UserService::join_room", err)]
    pub async fn join_room(&self, user_id: Uuid, room_id: Uuid) -> Result<(), UserServiceError> {
        // Single query: SET room = (SELECT id FROM room WHERE public_id = room_id)
        // WHERE public_id = user_id AND EXISTS (SELECT ... room WHERE public_id = room_id)
        // Resolves the internal integer FK and validates room existence atomically.
        let room_id_subquery = || {
            Query::select()
                .column(RoomColumn::Id)
                .from(RoomEntity)
                .and_where(RoomColumn::PublicId.eq(room_id))
                .to_owned()
        };

        let result = UserEntity::update_many()
            .col_expr(
                UserColumn::Room,
                Expr::SubQuery(
                    None,
                    Box::new(SubQueryStatement::SelectStatement(room_id_subquery())),
                ),
            )
            .filter(UserColumn::PublicId.eq(user_id))
            .filter(Expr::exists(room_id_subquery())) // Ensure the room exists, otherwise no rows will be updated
            .filter(
                // Skip the update if the user is already in this room (idempotency)
                Condition::any()
                    .add(UserColumn::Room.is_null())
                    .add(UserColumn::Room.not_in_subquery(room_id_subquery())),
            )
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            // Could be: already in room (idempotent, not an error) or user/room not found.
            let already_in_room = UserEntity::find()
                .join(JoinType::InnerJoin, UserRelation::Room.def())
                .filter(UserColumn::PublicId.eq(user_id))
                .filter(RoomColumn::PublicId.eq(room_id))
                .count(&self.db)
                .await?
                > 0;

            if already_in_room {
                debug!("join_room: user is already in room, no-op");
                return Ok(());
            }

            warn!("join_room failed: user or room not found");
            return Err(UserServiceError::RoomOrUserNotFound { user_id, room_id });
        }

        // Invalidate the user's cache entry since their room association has changed
        self.cache.users.invalidate(&user_id).await;

        // Invalidate users_in_room cache for the room they joined
        self.cache.users_in_room.invalidate(&room_id).await;

        // Publish user joined event to event bus
        self.event_bus
            .room
            .publish(RoomEvent::UserEntered { room_id, user_id });

        Ok(())
    }
}

/// Errors that can occur in UserService operations.
#[derive(Error, Debug, Clone)]
pub enum UserServiceError {
    #[error("Failed to add user: {0}")]
    UserNotAdded(String),

    #[error("User not found with id: {0}")]
    UserNotFound(Uuid),

    #[error("Room not found with id: {0}")]
    RoomNotFound(Uuid),

    #[error("User {user_id} or room {room_id} not found")]
    RoomOrUserNotFound { user_id: Uuid, room_id: Uuid },

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
            status: row.status.and_then(|s| {
                s.parse::<Status>()
                    .inspect_err(|e| {
                        error!(
                            raw_status = %s,
                             error = %e,
                            "Failed to parse user status"
                        )
                    })
                    .ok()
            }),
            room: row.room,
            last_seen: row.last_seen,
        }
    }
}

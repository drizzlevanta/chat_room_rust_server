use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// GraphQL output type for a user.
#[derive(SimpleObject, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub status: Option<UserStatus>,
    pub room: Option<Uuid>,
    pub last_seen: Option<DateTime<Utc>>,
}

impl From<domain::user::User> for User {
    fn from(u: domain::user::User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            status: u.status.map(UserStatus::from),
            room: u.room,
            last_seen: u.last_seen,
        }
    }
}

/// GraphQL enum mirroring domain::user::Status.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum UserStatus {
    Online,
    Offline,
    Away,
}

impl From<domain::user::Status> for UserStatus {
    fn from(s: domain::user::Status) -> Self {
        match s {
            domain::user::Status::Online => Self::Online,
            domain::user::Status::Offline => Self::Offline,
            domain::user::Status::Away => Self::Away,
        }
    }
}

impl From<UserStatus> for domain::user::Status {
    fn from(s: UserStatus) -> Self {
        match s {
            UserStatus::Online => Self::Online,
            UserStatus::Offline => Self::Offline,
            UserStatus::Away => Self::Away,
        }
    }
}

/// GraphQL input for creating a user.
#[derive(InputObject)]
pub struct CreateUserInput {
    pub name: String,
    /// Optional room public ID to place the user into on creation.
    pub room: Option<Uuid>,
}

/// GraphQL input for updating a user's status.
#[derive(InputObject)]
pub struct UpdateUserStatusInput {
    pub user_id: Uuid,
    pub status: UserStatus,
}

/// GraphQL output type for a user status change event.
#[derive(SimpleObject, Clone)]
pub struct UserStatusChanged {
    pub user_id: Uuid,
    pub status: UserStatus,
}

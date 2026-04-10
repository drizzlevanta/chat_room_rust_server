use async_graphql::{SimpleObject, Union};
use service::message_service::MessageServiceError;
use service::room_service::RoomServiceError;
use service::user_service::UserServiceError;
use tracing::error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

/// Concealed internal error — no service details leak to the client.
#[derive(SimpleObject, Debug, Clone)]
pub struct InternalError {
    /// Always `"Internal server error"`.
    pub message: String,
}

impl InternalError {
    pub fn new() -> Self {
        Self {
            message: "Internal server error".into(),
        }
    }
}

/// Returned when a mutation requires the `Idempotency-Key` header but it was
/// missing or not a valid UUID.
#[derive(SimpleObject, Debug, Clone)]
pub struct MissingIdempotencyKeyError {
    pub message: String,
}

impl MissingIdempotencyKeyError {
    pub fn new() -> Self {
        Self {
            message: "Missing or invalid Idempotency-Key header. Provide a valid UUID.".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Message errors
// ---------------------------------------------------------------------------

#[derive(SimpleObject, Debug, Clone)]
pub struct MessageTooLongError {
    pub message: String,
    pub max_length: usize,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserNotFoundInRoomError {
    pub message: String,
    pub user_id: Uuid,
    pub room_id: Uuid,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct RoomNotFoundError {
    pub message: String,
    pub room_id: Uuid,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct MessageNotFoundError {
    pub message: String,
    pub message_id: Uuid,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct RateLimitedError {
    pub message: String,
    pub max_requests: u32,
}

#[derive(Union, Debug)]
pub enum MessageError {
    MessageTooLong(MessageTooLongError),
    UserNotFoundInRoom(UserNotFoundInRoomError),
    RoomNotFound(RoomNotFoundError),
    MessageNotFound(MessageNotFoundError),
    RateLimited(RateLimitedError),
    MissingIdempotencyKey(MissingIdempotencyKeyError),
    InternalError(InternalError),
}

// ---------------------------------------------------------------------------
// Room errors
// ---------------------------------------------------------------------------

#[derive(SimpleObject, Debug, Clone)]
pub struct MaxCapacityExceededError {
    pub message: String,
    pub requested_capacity: u32,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct RoomNotAddedError {
    pub message: String,
}

#[derive(Union, Debug)]
pub enum RoomError {
    RoomNotFound(RoomNotFoundError),
    MaxCapacityExceeded(MaxCapacityExceededError),
    RoomNotAdded(RoomNotAddedError),
    MissingIdempotencyKey(MissingIdempotencyKeyError),
    InternalError(InternalError),
}

// ---------------------------------------------------------------------------
// User errors
// ---------------------------------------------------------------------------

#[derive(SimpleObject, Debug, Clone)]
pub struct UserNotFoundError {
    pub message: String,
    pub user_id: Uuid,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserNotAddedError {
    pub message: String,
    pub name: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserAlreadyExistsError {
    pub message: String,
    pub name: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserStatusUpdateFailedError {
    pub message: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserOrRoomNotFoundError {
    pub message: String,
    pub user_id: Uuid,
    pub room_id: Uuid,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct InvalidStatusError {
    pub message: String,
}

#[derive(Union, Debug)]
pub enum UserError {
    UserNotFound(UserNotFoundError),
    RoomNotFound(RoomNotFoundError),
    UserNotAdded(UserNotAddedError),
    UserAlreadyExists(UserAlreadyExistsError),
    UserStatusUpdateFailed(UserStatusUpdateFailedError),
    InvalidStatus(InvalidStatusError),
    MissingIdempotencyKey(MissingIdempotencyKeyError),
    UserOrRoomNotFound(UserOrRoomNotFoundError),
    InternalError(InternalError),
}

// ---------------------------------------------------------------------------
// From impls — service errors → GraphQL error unions
//
// Each error union (`MessageError`, `RoomError`, `UserError`) has a single
// `From<ServiceError>` impl that maps service errors to typed GraphQL error
// variants. Database errors are logged and concealed behind `InternalError`.
//
// Result unions use `#[graphql(flatten)]` on the error variant, so the
// conversion chain is: `ServiceError` → `DomainError` → `ResultUnion::Error`.
// ---------------------------------------------------------------------------

impl From<MessageServiceError> for MessageError {
    fn from(err: MessageServiceError) -> Self {
        match err {
            MessageServiceError::MessageTooLong(max) => {
                Self::MessageTooLong(MessageTooLongError {
                    message: err.to_string(),
                    max_length: max,
                })
            }
            MessageServiceError::UserNotFoundInRoom { user_id, room_id } => {
                Self::UserNotFoundInRoom(UserNotFoundInRoomError {
                    message: err.to_string(),
                    user_id,
                    room_id,
                })
            }
            MessageServiceError::RoomNotFound(id) => Self::RoomNotFound(RoomNotFoundError {
                message: err.to_string(),
                room_id: id,
            }),
            MessageServiceError::MessageNotFound(id) => {
                Self::MessageNotFound(MessageNotFoundError {
                    message: err.to_string(),
                    message_id: id,
                })
            }
            MessageServiceError::RateLimited(max) => Self::RateLimited(RateLimitedError {
                message: err.to_string(),
                max_requests: max,
            }),
            MessageServiceError::DatabaseError(ref inner) => {
                error!(error = %inner, "database error in message service");
                Self::InternalError(InternalError::new())
            }
        }
    }
}

impl From<RoomServiceError> for RoomError {
    fn from(err: RoomServiceError) -> Self {
        match err {
            RoomServiceError::RoomNotFound(id) => Self::RoomNotFound(RoomNotFoundError {
                message: err.to_string(),
                room_id: id,
            }),
            RoomServiceError::MaxCapacityExceeded(cap) => {
                Self::MaxCapacityExceeded(MaxCapacityExceededError {
                    message: err.to_string(),
                    requested_capacity: cap,
                })
            }
            RoomServiceError::RoomNotAdded(ref reason) => {
                error!(reason = %reason, "failed to add room");
                Self::RoomNotAdded(RoomNotAddedError {
                    message: "Failed to add room".into(),
                })
            }
            RoomServiceError::DatabaseError(ref inner) => {
                error!(error = %inner, "database error in room service");
                Self::InternalError(InternalError::new())
            }
        }
    }
}

impl From<UserServiceError> for UserError {
    fn from(err: UserServiceError) -> Self {
        match err {
            UserServiceError::UserNotFound(id) => Self::UserNotFound(UserNotFoundError {
                message: err.to_string(),
                user_id: id,
            }),
            UserServiceError::RoomNotFound(id) => Self::RoomNotFound(RoomNotFoundError {
                message: err.to_string(),
                room_id: id,
            }),
            UserServiceError::UserAlreadyExists(ref name) => {
                Self::UserAlreadyExists(UserAlreadyExistsError {
                    message: err.to_string(),
                    name: name.clone(),
                })
            }
            UserServiceError::InvalidStatus(_) => Self::InvalidStatus(InvalidStatusError {
                message: err.to_string(),
            }),
            UserServiceError::UserNotAdded(ref name) => {
                error!(name = %name, "failed to add user");
                Self::UserNotAdded(UserNotAddedError {
                    message: format!("Failed to add user '{name}'"),
                    name: name.clone(),
                })
            }
            UserServiceError::UserStatusUpdateFailed(ref reason) => {
                error!(reason = %reason, "failed to update user status");
                Self::UserStatusUpdateFailed(UserStatusUpdateFailedError {
                    message: err.to_string(),
                })
            }
            UserServiceError::RoomOrUserNotFound { user_id, room_id } => {
                Self::UserOrRoomNotFound(UserOrRoomNotFoundError {
                    message: err.to_string(),
                    user_id,
                    room_id,
                })
            }
            UserServiceError::DatabaseError(ref inner) => {
                error!(error = %inner, "database error in user service");
                Self::InternalError(InternalError::new())
            }
        }
    }
}

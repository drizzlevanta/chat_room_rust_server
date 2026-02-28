use async_graphql::{SimpleObject, Union};
use domain::constants::MAX_MESSAGE_LENGTH;
use service::message_service::MessageServiceError;
use service::room_service::RoomServiceError;
use service::user_service::UserServiceError;
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

#[derive(Union, Debug)]
pub enum MessageError {
    MessageTooLong(MessageTooLongError),
    UserNotFoundInRoom(UserNotFoundInRoomError),
    RoomNotFound(RoomNotFoundError),
    MessageNotFound(MessageNotFoundError),
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
            MessageServiceError::MessageTooLong => Self::MessageTooLong(MessageTooLongError {
                message: err.to_string(),
                max_length: MAX_MESSAGE_LENGTH,
            }),
            MessageServiceError::UserNotFoundinRoom { user_id, room_id } => {
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
            MessageServiceError::DatabaseError(ref inner) => {
                eprintln!("Database error in message service: {inner}");
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
                eprintln!("Failed to add room: {reason}");
                Self::RoomNotAdded(RoomNotAddedError {
                    message: "Failed to add room".into(),
                })
            }
            RoomServiceError::DatabaseError(ref inner) => {
                eprintln!("Database error in room service: {inner}");
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
                eprintln!("Failed to add user: {name}");
                Self::UserNotAdded(UserNotAddedError {
                    message: format!("Failed to add user '{name}'"),
                    name: name.clone(),
                })
            }
            UserServiceError::UserStatusUpdateFailed(ref reason) => {
                eprintln!("Failed to update user status: {reason}");
                Self::UserStatusUpdateFailed(UserStatusUpdateFailedError {
                    message: "Failed to update user status".into(),
                })
            }
            UserServiceError::DatabaseError(ref inner) => {
                eprintln!("Database error in user service: {inner}");
                Self::InternalError(InternalError::new())
            }
        }
    }
}

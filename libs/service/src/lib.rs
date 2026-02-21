use std::sync::Arc;

use sea_orm::DatabaseConnection;

pub mod mappers;
pub mod message_service;
pub mod room_service;
pub mod user_service;

use message_service::MessageService;
use room_service::RoomService;
use user_service::UserService;

/// Holds all service instances. Wrap in `Arc` to share across API layers.
pub struct ServiceContainer {
    pub room: RoomService,
    pub user: UserService,
    pub message: MessageService,
}

impl ServiceContainer {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            room: RoomService::new(db.clone()),
            user: UserService::new(db.clone()),
            message: MessageService::new(db),
        }
    }

    /// Convenience: create and wrap in Arc in one call.
    pub fn new_shared(db: DatabaseConnection) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

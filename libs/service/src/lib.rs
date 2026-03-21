use std::sync::Arc;

use sea_orm::DatabaseConnection;

pub mod cache;
pub mod event_bus;
pub mod mappers;
pub mod message_service;
pub mod room_service;
pub mod user_service;

use cache::ChatCache;
use message_service::MessageService;
use room_service::RoomService;
use user_service::UserService;

use crate::event_bus::EventBus;

/// Holds all service instances. Wrap in `Arc` to share across API layers.
pub struct ServiceContainer {
    pub room: RoomService,
    pub user: UserService,
    pub message: MessageService,
    pub cache: ChatCache,
    pub event_bus: EventBus,
}

impl ServiceContainer {
    pub fn new(db: DatabaseConnection) -> Self {
        let cache = ChatCache::new();
        Self {
            room: RoomService::new(db.clone(), cache.clone()),
            user: UserService::new(db.clone(), cache.clone()),
            message: MessageService::new(db, cache.clone()),
            cache,
            event_bus: EventBus::new(),
        }
    }

    /// Convenience: create and wrap in Arc in one call.
    pub fn new_shared(db: DatabaseConnection) -> Arc<Self> {
        Arc::new(Self::new(db))
    }
}

use std::sync::Arc;

use domain::config::AppConfig;
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
    pub config: Arc<AppConfig>,
}

impl ServiceContainer {
    pub fn new(db: DatabaseConnection, config: Arc<AppConfig>) -> Self {
        let cache = ChatCache::new(&config.cache);
        let event_bus = EventBus::new(&config.event_bus);
        Self {
            room: RoomService::new(db.clone(), cache.clone(), event_bus.clone(), config.clone()),
            user: UserService::new(db.clone(), cache.clone(), event_bus.clone()),
            message: MessageService::new(db, cache.clone(), event_bus.clone(), config.clone()),
            cache,
            event_bus,
            config,
        }
    }

    /// Convenience: create and wrap in Arc in one call.
    pub fn new_shared(db: DatabaseConnection, config: Arc<AppConfig>) -> Arc<Self> {
        Arc::new(Self::new(db, config))
    }
}

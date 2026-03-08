use std::time::Duration;

use domain::message::Message;
use domain::room::Room;
use domain::user::User;
use moka::future::Cache;
use uuid::Uuid;

/// Centralized in-memory cache for the chat application.
///
/// Holds separate moka caches for rooms, users, and messages.
/// Cheap to clone (internally Arc-wrapped) — share freely across services.
#[derive(Clone)]
pub struct ChatCache {
    /// Single room by public ID.
    pub rooms: Cache<Uuid, Room>,
    /// All rooms list (unit key since there's only one list).
    pub all_rooms: Cache<(), Vec<Room>>,
    /// Single user by public ID.
    pub users: Cache<Uuid, User>,
    /// Users in a room, keyed by room public ID.
    pub users_in_room: Cache<Uuid, Vec<User>>,
    /// Latest messages in a room, keyed by room public ID.
    pub latest_messages: Cache<Uuid, Vec<Message>>,
}

impl ChatCache {
    pub fn new() -> Self {
        Self {
            rooms: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(300)) // 5 min
                .build(),
            all_rooms: Cache::builder()
                .max_capacity(1) // Only one list of all rooms
                .time_to_live(Duration::from_secs(120)) // 2 min
                .build(),
            users: Cache::builder()
                .max_capacity(5_000)
                .time_to_live(Duration::from_secs(120)) // 2 min
                .build(),
            users_in_room: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(60)) // 1 min
                .build(),
            latest_messages: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(30)) // 30 sec
                .build(),
        }
    }

    /// Invalidate all room-related caches (single room + all-rooms list).
    pub async fn invalidate_room(&self, room_id: &Uuid) {
        self.rooms.invalidate(room_id).await;
        self.all_rooms.invalidate(&()).await;
    }

    /// Invalidate all rooms list cache
    /// (used when we know a room was updated but don't have the specific ID, e.g. after bulk updates).
    pub async fn invalidate_all_rooms(&self) {
        self.all_rooms.invalidate(&()).await;
    }

    /// Invalidate all user-related caches for a given user and their room.
    pub async fn invalidate_user(&self, user_id: &Uuid, room_id: Option<&Uuid>) {
        self.users.invalidate(user_id).await;
        if let Some(rid) = room_id {
            self.users_in_room.invalidate(rid).await;
        }
    }

    /// Invalidate message caches for a room.
    pub async fn invalidate_messages(&self, room_id: &Uuid) {
        self.latest_messages.invalidate(room_id).await;
    }
}

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use domain::config::CacheConfig;
use domain::message::Message;
use domain::pagination::CursorPage;
use domain::room::Room;
use domain::user::User;
use moka::future::Cache;
use uuid::Uuid;

/// Compound key for the idempotency cache.
/// Scoped to a user to prevent collisions.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdempotencyKey {
    pub user_id: Uuid,
    pub key: Uuid,
}

/// Compound key for the typing indicators debounce cache.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypingIndicatorKey {
    pub room_id: Uuid,
    pub user_id: Uuid,
}

/// Centralized in-memory cache for the chat application.
///
/// Holds separate moka caches for rooms, users, and messages.
/// Cheap to clone (internally Arc-wrapped).
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
    /// The "latest" messages are defined as the most recent N messages (configured via `cache.latest_messages.limit`).
    /// If client requests more, the service will bypass the cache and query the database directly.
    pub latest_messages: Cache<Uuid, CursorPage<Message>>,
    /// Idempotency cache for message creation, keyed by (user_id, idempotency_key).
    pub idempotency_message: Cache<IdempotencyKey, Message>,
    /// Idempotency cache for room creation. Keyed by idempotency key only since rooms aren't user-scoped.
    pub idempotency_room: Cache<Uuid, Room>,
    /// Idempotency cache for user creation. Keyed by idempotency key.
    pub idempotency_user: Cache<Uuid, User>,

    /// Typing indicator debounce guard, keyed by (room_id, user_id).
    pub typing_indicators: Cache<TypingIndicatorKey, ()>,

    /// Per-user rate-limit counters. Each entry holds an atomic request count
    /// that expires after the configured rate-limit window.
    pub rate_limits: Cache<Uuid, Arc<AtomicU32>>,

    /// Maximum number of messages that can be served from the latest-messages cache.
    /// Requests for more than this bypass the cache and hit the database directly.
    pub latest_messages_cache_limit: u64,

    /// Maximum number of write operations a single user can perform per rate-limit window.
    pub rate_limit_max_requests: u32,
}

impl ChatCache {
    pub fn new(config: &CacheConfig) -> Self {
        Self {
            rooms: Cache::builder()
                .max_capacity(config.room.capacity)
                .time_to_live(Duration::from_secs(config.room.ttl_secs))
                .build(),
            all_rooms: Cache::builder()
                .max_capacity(config.all_rooms.capacity)
                .time_to_live(Duration::from_secs(config.all_rooms.ttl_secs))
                .build(),
            users: Cache::builder()
                .max_capacity(config.user.capacity)
                .time_to_live(Duration::from_secs(config.user.ttl_secs))
                .build(),
            users_in_room: Cache::builder()
                .max_capacity(config.users_in_room.capacity)
                .time_to_live(Duration::from_secs(config.users_in_room.ttl_secs))
                .build(),
            latest_messages: Cache::builder()
                .max_capacity(config.latest_messages.capacity)
                .time_to_live(Duration::from_secs(config.latest_messages.ttl_secs))
                .build(),
            idempotency_message: Cache::builder()
                .max_capacity(config.idempotency.capacity)
                .time_to_live(Duration::from_secs(config.idempotency.ttl_secs))
                .build(),
            idempotency_room: Cache::builder()
                .max_capacity(config.idempotency.capacity)
                .time_to_live(Duration::from_secs(config.idempotency.ttl_secs))
                .build(),
            idempotency_user: Cache::builder()
                .max_capacity(config.idempotency.capacity)
                .time_to_live(Duration::from_secs(config.idempotency.ttl_secs))
                .build(),
            typing_indicators: Cache::builder()
                .max_capacity(config.typing_indicators.capacity)
                .time_to_live(Duration::from_secs(config.typing_indicators.ttl_secs))
                .build(),
            rate_limits: Cache::builder()
                .max_capacity(config.rate_limit.capacity)
                .time_to_live(Duration::from_secs(config.rate_limit.window_secs))
                .build(),
            latest_messages_cache_limit: config.latest_messages.limit,
            rate_limit_max_requests: config.rate_limit.max_requests,
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

    /// Check whether a user has exceeded the per-window rate limit.
    ///
    /// Returns `Ok(())` if the request is allowed, or `Err(())` if the user
    /// has hit the maximum number of requests for the current window.
    pub async fn check_rate_limit(&self, user_id: Uuid) -> Result<(), ()> {
        let counter = self
            .rate_limits
            .get_with(user_id, async { Arc::new(AtomicU32::new(0)) })
            .await;
        // Atomically increment the counter, returns the previous value and check if it exceeds the limit.
        let prev = counter.fetch_add(1, Ordering::Relaxed);
        if prev >= self.rate_limit_max_requests {
            Err(())
        } else {
            Ok(())
        }
    }
}

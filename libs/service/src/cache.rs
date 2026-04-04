use std::time::Duration;

use domain::message::Message;
use domain::pagination::CursorPage;
use domain::room::Room;
use domain::user::User;
use moka::future::Cache;
use uuid::Uuid;

const ROOM_CACHE_CAPACITY: u64 = 500;
const ROOM_CACHE_TTL_SECS: u64 = 300;
const ALL_ROOMS_CACHE_CAPACITY: u64 = 1;
const ALL_ROOMS_CACHE_TTL_SECS: u64 = 120;
const USER_CACHE_CAPACITY: u64 = 5_000;
const USER_CACHE_TTL_SECS: u64 = 120;
const USERS_IN_ROOM_CACHE_CAPACITY: u64 = 500;
const USERS_IN_ROOM_CACHE_TTL_SECS: u64 = 60;
const LATEST_MESSAGES_CACHE_CAPACITY: u64 = 500;
const LATEST_MESSAGES_CACHE_TTL_SECS: u64 = 30;
/// Maximum number of messages that can be served from the latest-messages cache.
/// Requests for more than this bypass the cache and hit the database directly.
pub const LATEST_MESSAGES_CACHE_LIMIT: u64 = 50;
const IDEMPOTENCY_CACHE_CAPACITY: u64 = 10_000;
const IDEMPOTENCY_CACHE_TTL_SECS: u64 = 300;

const TYPING_INDICATORS_CACHE_CAPACITY: u64 = 10_000;
const TYPING_INDICATORS_CACHE_TTL_SECS: u64 = 5; // Typing indicators are very ephemeral, so we use a short TTL to prevent stale data.

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
    /// The "latest" messages are defined as the most recent `LATEST_MESSAGES_CACHE_LIMIT` messages.
    /// If client requests more, the service will bypass the cache and query the database directly.
    // pub latest_messages: Cache<Uuid, Vec<Message>>,
    pub latest_messages: Cache<Uuid, CursorPage<Message>>,
    /// Idempotency cache for message creation, keyed by (user_id, idempotency_key).
    pub idempotency_message: Cache<IdempotencyKey, Message>,
    /// Idempotency cache for room creation. Keyed by idempotency key only since rooms aren't user-scoped.
    pub idempotency_room: Cache<Uuid, Room>,
    /// Idempotency cache for user creation. Keyed by idempotency key.
    pub idempotency_user: Cache<Uuid, User>,

    /// Typing indicator debounce guard, keyed by (room_id, user_id).
    pub typing_indicators: Cache<TypingIndicatorKey, ()>,
}

impl ChatCache {
    pub fn new() -> Self {
        Self {
            rooms: Cache::builder()
                .max_capacity(ROOM_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(ROOM_CACHE_TTL_SECS))
                .build(),
            all_rooms: Cache::builder()
                .max_capacity(ALL_ROOMS_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(ALL_ROOMS_CACHE_TTL_SECS))
                .build(),
            users: Cache::builder()
                .max_capacity(USER_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(USER_CACHE_TTL_SECS))
                .build(),
            users_in_room: Cache::builder()
                .max_capacity(USERS_IN_ROOM_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(USERS_IN_ROOM_CACHE_TTL_SECS))
                .build(),
            latest_messages: Cache::builder()
                .max_capacity(LATEST_MESSAGES_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(LATEST_MESSAGES_CACHE_TTL_SECS))
                .build(),
            idempotency_message: Cache::builder()
                .max_capacity(IDEMPOTENCY_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(IDEMPOTENCY_CACHE_TTL_SECS))
                .build(),
            idempotency_room: Cache::builder()
                .max_capacity(IDEMPOTENCY_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(IDEMPOTENCY_CACHE_TTL_SECS))
                .build(),
            idempotency_user: Cache::builder()
                .max_capacity(IDEMPOTENCY_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(IDEMPOTENCY_CACHE_TTL_SECS))
                .build(),
            typing_indicators: Cache::builder()
                .max_capacity(TYPING_INDICATORS_CACHE_CAPACITY)
                .time_to_live(Duration::from_secs(TYPING_INDICATORS_CACHE_TTL_SECS))
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

use serde::Deserialize;

/// Top-level application configuration.
///
/// All fields have sensible defaults so the server can start with an empty
/// (or missing) config file. Load from a TOML file with [`AppConfig::from_toml`],
/// or just use [`AppConfig::default()`].
#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub cache: CacheConfig,
    pub event_bus: EventBusConfig,
    pub message: MessageConfig,
    pub room: RoomConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            event_bus: EventBusConfig::default(),
            message: MessageConfig::default(),
            room: RoomConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct CacheConfig {
    pub room: CacheEntry,
    pub all_rooms: CacheEntry,
    pub user: CacheEntry,
    pub users_in_room: CacheEntry,
    pub latest_messages: LatestMessagesCacheConfig,
    pub idempotency: CacheEntry,
    pub typing_indicators: CacheEntry,
    pub rate_limit: RateLimitConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            room: CacheEntry {
                capacity: 500,
                ttl_secs: 300,
            },
            all_rooms: CacheEntry {
                capacity: 1,
                ttl_secs: 120,
            },
            user: CacheEntry {
                capacity: 5_000,
                ttl_secs: 120,
            },
            users_in_room: CacheEntry {
                capacity: 500,
                ttl_secs: 60,
            },
            latest_messages: LatestMessagesCacheConfig::default(),
            idempotency: CacheEntry {
                capacity: 10_000,
                ttl_secs: 300,
            },
            typing_indicators: CacheEntry {
                capacity: 10_000,
                ttl_secs: 5,
            },
            rate_limit: RateLimitConfig::default(),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct CacheEntry {
    pub capacity: u64,
    pub ttl_secs: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct LatestMessagesCacheConfig {
    pub capacity: u64,
    pub ttl_secs: u64,
    /// Maximum number of messages served from the cache.
    /// Requests for more than this bypass the cache and hit the database.
    pub limit: u64,
}

impl Default for LatestMessagesCacheConfig {
    fn default() -> Self {
        Self {
            capacity: 500,
            ttl_secs: 30,
            limit: 50,
        }
    }
}

// ---------------------------------------------------------------------------
// Rate limiting
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RateLimitConfig {
    pub capacity: u64,
    pub window_secs: u64,
    pub max_requests: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            capacity: 100_000,
            window_secs: 10,
            max_requests: 5,
        }
    }
}

// ---------------------------------------------------------------------------
// Event bus
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct EventBusConfig {
    pub default_capacity: usize,
    pub message_capacity: usize,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            default_capacity: 1024,
            message_capacity: 2048,
        }
    }
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct MessageConfig {
    pub max_length: usize,
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self { max_length: 2_000 }
    }
}

// ---------------------------------------------------------------------------
// Room
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RoomConfig {
    pub default_capacity: u32,
    pub max_capacity: u32,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            default_capacity: 100,
            max_capacity: 1_000,
        }
    }
}

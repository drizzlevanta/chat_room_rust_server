use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Domain representation of a message
pub struct Message {
    pub id: Uuid, // Public ID of the message
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub sender: Uuid, // Public ID of the user
    pub room: Uuid,   // Public ID of the room
}

impl Message {
    pub fn new(content: String, created_at: DateTime<Utc>, sender: Uuid, room: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at,
            content,
            sender,
            room,
        }
    }
}

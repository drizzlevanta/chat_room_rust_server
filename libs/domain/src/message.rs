use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Domain representation of a message
#[derive(Clone, Debug)]
pub struct Message {
    pub id: Uuid, // Public ID of the message
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub sender: Uuid, // Public ID of the user
    pub room: Uuid,   // Public ID of the room
}

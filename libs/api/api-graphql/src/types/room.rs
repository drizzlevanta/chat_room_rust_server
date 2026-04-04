use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use domain::events::TypingEvent;
use domain::room::Room as DomainRoom;
use uuid::Uuid;

/// GraphQL output type for a room.
#[derive(SimpleObject, Clone)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: u32,
}

impl From<DomainRoom> for Room {
    fn from(r: DomainRoom) -> Self {
        Self {
            id: r.id,
            name: r.name,
            description: r.description,
            capacity: r.capacity,
        }
    }
}

/// GraphQL input for creating a room.
#[derive(InputObject)]
pub struct CreateRoomInput {
    pub name: String,
    pub capacity: Option<u32>,
}

/// GraphQL output type for a typing indicator event.
#[derive(SimpleObject, Clone)]
pub struct TypingIndicator {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub is_typing: bool,
    pub timestamp: DateTime<Utc>,
}

impl From<TypingEvent> for TypingIndicator {
    fn from(e: TypingEvent) -> Self {
        Self {
            user_id: e.user_id,
            room_id: e.room_id,
            is_typing: e.is_typing,
            timestamp: e.timestamp,
        }
    }
}

/// GraphQL output type for a user-entered-room event.
#[derive(SimpleObject, Clone)]
pub struct UserEnteredRoom {
    pub room_id: Uuid,
    pub user_id: Uuid,
}

/// GraphQL output type for a user-left-room event.
#[derive(SimpleObject, Clone)]
pub struct UserLeftRoom {
    pub room_id: Uuid,
    pub user_id: Uuid,
}

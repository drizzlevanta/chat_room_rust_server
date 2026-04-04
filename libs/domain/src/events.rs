use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::message::Message;
use crate::room::Room;
use crate::user::{Status, User};

/// A typing indicator event — ephemeral, never persisted.
#[derive(Clone, Debug)]
pub struct TypingEvent {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub is_typing: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub enum UserEvent {
    Added(User),
    Removed(Uuid), // user ID
    StatusChanged { user_id: Uuid, status: Status },
}

#[derive(Clone, Debug)]
pub enum RoomEvent {
    Added(Room),   // room ID
    Removed(Uuid), // room ID
    UserEntered { room_id: Uuid, user_id: Uuid },
    UserLeft { room_id: Uuid, user_id: Uuid },
    UserTyping(TypingEvent),
}

#[derive(Clone, Debug)]
pub enum MessageEvent {
    Sent(Message),
    Edited(Message),
    Deleted { message_id: Uuid, room_id: Uuid },
}

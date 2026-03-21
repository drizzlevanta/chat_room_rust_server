use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::message::Message;
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
    Typing(TypingEvent),
}

#[derive(Clone, Debug)]
pub enum RoomEvent {
    Added(Uuid),   // room ID
    Removed(Uuid), // room ID
    UserEntered { room_id: Uuid, user_id: Uuid },
    UserLeft { room_id: Uuid, user_id: Uuid },
}

#[derive(Clone, Debug)]
pub enum MessageEvent {
    Sent(Message),
    Edited(Message),
    Deleted(Uuid), // message ID
}

/// Central enum of all domain events routed through the event bus.
#[derive(Clone, Debug)]
pub enum DomainEvent {
    Typing(TypingEvent),
    MessageSent(Message),
    UserStatusChanged {
        user_id: Uuid,
        room_id: Option<Uuid>,
        status: Status,
    },
}

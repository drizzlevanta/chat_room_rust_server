use tokio::sync::broadcast;

use crate::{rooms::room::Room, users::user::User};

#[derive(Clone)]
pub enum UserEvent{
    UserAdded(User),
    UserRemoved(String), // username
}

#[derive(Clone)]
pub enum RoomEvent{
    RoomAdded(Room),
    RoomRemoved(String), // room name
    UserEntered(String, User), // room name, user
    UserLeft(String, User), // room name, user
}

pub struct EventBus{
    pub user: broadcast::Sender<UserEvent>,
    pub room: broadcast::Sender<RoomEvent>,
}

impl EventBus{
    // Initialize a new EventBus
    pub fn new(buffer: usize)->Self{
        let (user_tx, _rx) = broadcast::channel(buffer);
        let (room_tx, _rx) = broadcast::channel(buffer);
        EventBus { user: user_tx, room: room_tx }
    }
}





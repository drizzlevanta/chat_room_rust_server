use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::{chats::message::Message, rooms::room::Room, users::user::{Status, User}};


pub struct ChatRoomState {
    pub users: Vec<User>,
    pub messages: Vec<Message>,
    pub rooms: Vec<Room>, 
}

impl ChatRoomState {
    pub fn new() -> Self {
        let default_users = vec![
            User {
                name: "Alice".to_string(),
                status: Status::Online,
                room: Some("General".to_string()),
            },
            User {
                name: "AliceInWonder".to_string(),
                status: Status::Online,
                room: Some("Anime".to_string()),
            },
            User {
                name: "Bob".to_string(),
                status: Status::Offline,
                room: None,
            },
        ];

        let default_messages = vec![
            Message {
                id: Uuid::new_v4(),
                content: "bacon v5".to_string(),
                timestamp: "2023-10-01T12:00:00Z".to_string(),
                room_name: "General".to_string(),
                sender: "Alice".to_string(),
            },
            Message {
                id: Uuid::new_v4(),
                content: "Bob said: How's it going?".to_string(),
                timestamp: "2023-10-01T12:05:00Z".to_string(),
                room_name: "General".to_string(),
                sender: "Bob".to_string(),
            },
        ];

        let default_rooms = vec![
            Room {
                name: "General".to_string(),
                capacity: 50,
            },
            Room {
                name: "Random".to_string(),
                capacity: 50,
            },
            Room {
                name: "Tech".to_string(),
                capacity:100
            },
            Room {
                name: "Gaming".to_string(),
                capacity:10
            },
            Room {
                name: "Anime".to_string(),
                capacity:5
            },
        ];
        ChatRoomState {
            users: default_users,
            messages: default_messages,
            rooms: default_rooms,
        }
    }
}

pub type SharedChatRoomState = Arc<RwLock<ChatRoomState>>;
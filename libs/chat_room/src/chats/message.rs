use async_graphql::{Context, SimpleObject};
use thiserror::Error;
use uuid::Uuid;

use crate::{rooms::room::Room, state::SharedChatRoomState};

#[derive(SimpleObject, Clone, Debug)]
pub struct Message{
    pub id: Uuid,
    pub timestamp: String,
    pub content: String,
    pub sender: String,
    pub room_name: String,
}

impl Message{
    pub async fn room(&self, ctx: &Context<'_>) -> Room {
        let state = ctx.data_unchecked::<SharedChatRoomState>();
        let state = state.read().await;
        state.rooms.iter().find(|r| r.name == self.room_name).cloned().unwrap_or_default()
    }
}

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Room not found on specified message {0}")]
    RoomNotFound(Uuid),
}

use async_graphql::{Context, Object, SimpleObject, Union};
use chrono::Utc;
use uuid::Uuid;

use crate::{chats::message::Message, shared::models::UserNotFound, state::SharedChatRoomState};

#[derive(Debug, SimpleObject, Clone)]
pub struct MessageNotFound {
    pub id: Uuid,
}

#[derive(Union)]
pub enum ChatsQueryResult{
    Message(Message),
    Error(MessageNotFound),
}

#[derive(Default)]
pub struct ChatsQuery;

#[Object]
impl ChatsQuery{
    async fn messages(&self, ctx: &Context<'_>)->Vec<Message>{
        let state=ctx.data_unchecked::<SharedChatRoomState>();
        let state=state.read().await;
        state.messages.clone()
    }

    async fn message_by_id(&self, ctx: &Context<'_>, id: Uuid) -> ChatsQueryResult {
        let state = ctx.data_unchecked::<SharedChatRoomState>();
        let state = state.read().await;
        match state.messages.iter().find(|m| m.id == id).cloned() {
            Some(m) => ChatsQueryResult::Message(m),
            None => ChatsQueryResult::Error(MessageNotFound { id }),
        }
    }
    
    async fn messages_by_user(&self, ctx: &Context<'_>, user_name: String) -> Vec<Message> {
        let state = ctx.data_unchecked::<SharedChatRoomState>();
        let state = state.read().await;
        state.messages.iter().filter(|m| m.sender == user_name).cloned().collect()
    }
}

#[derive(Debug, SimpleObject, Clone)]
pub struct RoomNotFound {
    pub room_name: String,
}


#[derive(Union)]
pub enum AddMessageResult{
    Message(Message),
    RoomNotFound(RoomNotFound),
    UserNotFound(UserNotFound),
}


#[derive(Default)]
pub struct ChatsMutation;

#[Object]
impl ChatsMutation {
    async fn add_message(&self, ctx: &Context<'_>, content: String, sender: String, room_name: String) -> AddMessageResult {
        // Validate room exists
        let state = ctx.data_unchecked::<SharedChatRoomState>();
        {
            let state=state.read().await;
            // Validate room exists
            if !state.rooms.iter().any(|r| r.name == room_name) {
                return AddMessageResult::RoomNotFound(RoomNotFound { room_name });
            }
    
            // Validate user exists
            if !state.users.iter().any(|u| u.name == sender) {
                return AddMessageResult::UserNotFound(UserNotFound { name: sender });
            }
        }

        // Construct a new message
        let new_message=Message{
            id: Uuid::new_v4(),
            content,
            sender,
            timestamp: Utc::now().to_rfc3339(),
            room_name,
        };

        // Aquire write lock after read lock is released
        let mut state=state.write().await;
        state.messages.push(new_message.clone());
        AddMessageResult::Message(new_message)
    }
}
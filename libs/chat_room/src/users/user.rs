use async_graphql::{ComplexObject, Context, Enum, InputObject, SimpleObject};

use crate::{chats::message::Message, state::SharedChatRoomState};

#[derive(SimpleObject, Clone, InputObject)]
#[graphql(input_name = "UserInput")]
#[graphql(complex)] 
pub struct User {
    #[graphql(validator(min_length=2))]
    pub name: String,
    pub status: Status,
    pub room: Option<String>,  // Optional field to indicate the room the user is in
}

#[ComplexObject]
impl User {
    async fn messages(&self, ctx:&Context<'_>)->Vec<Message>{
        let state=ctx.data_unchecked::<SharedChatRoomState>();
        let state=state.read().await;
        let messages=state.messages.iter().filter(|m| m.sender == self.name).cloned().collect::<Vec<_>>();
        println!("User: {}, Messages: {:?}", self.name, messages);
        messages
    }
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status{
    Online,
    Offline,
    Away,
}
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject};

use crate::{chats::message::Message, state::SharedChatRoomState, users::user::User};

#[derive(SimpleObject, Clone, Default, InputObject)]
#[graphql(input_name = "RoomInput")]
#[graphql(complex)]
pub struct Room {
    #[graphql(validator(min_length=2))]
    pub name: String,
    #[graphql(validator(minimum = 1))]
    pub capacity: usize,
}

#[ComplexObject]
impl Room {
    async fn users(&self, ctx:&Context<'_>)->Vec<User>{
        let state=ctx.data_unchecked::<SharedChatRoomState>();
        let state=state.read().await;
        state.users.iter().filter(|u| u.room.as_deref()==Some(&self.name)).cloned().collect()
    }

    async fn messages(&self, ctx: &Context<'_>)->Vec<Message>{
        let state=ctx.data_unchecked::<SharedChatRoomState>();
        let state=state.read().await;
        state.messages.iter().filter(|m|m.room_name==self.name).cloned().collect()
    }
}

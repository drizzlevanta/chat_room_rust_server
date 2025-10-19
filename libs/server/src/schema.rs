use std::sync::Arc;

use async_graphql::{MergedObject, MergedSubscription, Schema};
use chat_room::{chats::chats_resolver::{ChatsMutation, ChatsQuery}, rooms::rooms_resolver::{RoomsMutation, RoomsQuery, RoomsSubscription}, shared::event_bus::EventBus, state::{ChatRoomState, SharedChatRoomState}, users::users_resolver::{UsersMutation, UsersQuery, UsersSubscription}};
use tokio::sync::RwLock;


#[derive(MergedObject, Default)]
pub struct QueryRoot(ChatsQuery, UsersQuery, RoomsQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(ChatsMutation, RoomsMutation, UsersMutation);

#[derive(MergedSubscription,Default)]
pub struct SubscriptionRoot(UsersSubscription, RoomsSubscription);

pub type AppSchema=Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn create_schema(event_bus: EventBus) -> AppSchema {
    let shared_chat_room_state:SharedChatRoomState = Arc::new(RwLock::new(ChatRoomState::new()));
    Schema::build(QueryRoot::default(), MutationRoot::default(), SubscriptionRoot::default()).data(shared_chat_room_state).data(event_bus).finish()
}
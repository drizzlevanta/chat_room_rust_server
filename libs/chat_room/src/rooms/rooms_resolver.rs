use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject, Subscription, Union};
use futures_util::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;

use crate::{rooms::room::Room, shared::event_bus::{EventBus, RoomEvent}, state::SharedChatRoomState, users::user::User};

#[derive(Default)]
pub struct RoomsQuery;

#[Object]
impl RoomsQuery {
    async fn rooms(&self, ctx: &Context<'_>) -> Vec<Room> {
        let state = ctx.data_unchecked::<SharedChatRoomState>();
        let state = state.read().await;
        
        state.rooms.clone()
    }
}

#[derive(Debug, SimpleObject, Clone)]
pub struct RoomAlreadyExists {
    pub room_name: String,
}

#[derive(Union)]
pub enum AddRoomResult {
    RoomAdded(Room),
    RoomAlreadyExists(RoomAlreadyExists),
}


#[derive(Default)]
pub struct RoomsMutation;

#[Object]
impl RoomsMutation {
    async fn add_room(&self, ctx: &Context<'_>, room_input: Room) -> AddRoomResult {
        let state = ctx.data_unchecked::<SharedChatRoomState>();

        //Check if room exists
        {
            // Scoping out the read lock to release it before acquiring the write lock
            let state = state.read().await;
            if state.rooms.iter().any(|room| room.name == room_input.name) {
                return AddRoomResult::RoomAlreadyExists(RoomAlreadyExists { room_name: room_input.name.clone() });
            }
        }
       
        // Acquire write lock after releasing read lock
        let mut state = state.write().await;

        let new_room = Room { name: room_input.name, capacity: room_input.capacity };
        state.rooms.push(new_room.clone());

        // Publish to event bus
        let event_bus=ctx.data_unchecked::<EventBus>();
        let _=event_bus.room.send(RoomEvent::RoomAdded(new_room.clone()));

        AddRoomResult::RoomAdded(new_room)
    }
}


#[derive(Default)]
pub struct RoomsSubscription;

#[Subscription]
impl RoomsSubscription {
    async fn room_added(&self, ctx: &Context<'_>) -> impl Stream<Item = Room> {
        let rx=ctx.data_unchecked::<EventBus>().room.subscribe();
        BroadcastStream::new(rx).filter_map(|event|async move{
            match event{
                Ok(RoomEvent::RoomAdded(room))=>Some(room),
                _=>None,
            }
        })
    }

    async fn user_entered(&self, ctx: &Context<'_>, room_name: String)->impl Stream<Item = User>{
        let rx=ctx.data_unchecked::<EventBus>().room.subscribe();
        let room_name=Arc::new(room_name); // using Arc to allow cloning into the async closure

        // The first `move` moves room_name into the closure, since borrowing across async boundary is not allowed. 
        // The closure keeps `room_name` around for every invocation.
        BroadcastStream::new(rx).filter_map(move |event|{
            let room_name=Arc::clone(&room_name);
            // second move gives the future ownership of the cloned room_name
            async move{
            match event{
                Ok(RoomEvent::UserEntered(name, user )) if name==*room_name=>Some(user), 
                _=>None,
            }
        }})
    }

    async fn user_left(&self, ctx: &Context<'_>, room_name: String)->impl Stream<Item=User>{
        let rx=ctx.data_unchecked::<EventBus>().room.subscribe();
        let room_name=Arc::new(room_name);

        BroadcastStream::new(rx).filter_map(move |event|{
            let room_name=Arc::clone(&room_name);
            async move{
                match event{
                    Ok(RoomEvent::UserLeft(name, user)) if name==*room_name=>Some(user),
                    _=>None,
                }
            }
        })
    }
}

use async_graphql::{Context, Object, SimpleObject, Subscription, Union};
use crate::{shared::{event_bus::{EventBus, RoomEvent, UserEvent}, models::UserNotFound}, state::SharedChatRoomState, users::user::User};
use futures_util::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;

#[derive(Default)]
pub struct UsersQuery;

#[Object]
impl UsersQuery {
    pub async fn users(&self, ctx: &Context<'_>) -> Vec<User> {
        let state=ctx.data_unchecked::<SharedChatRoomState>();
        let state=state.read().await;
        state.users.clone()
    }
}

#[derive(Debug, SimpleObject, Clone)]
pub struct UserAlreadyExists {
    pub user_name: String,
}


#[derive(Union)]
pub enum AddUserResult {
    UserAdded(User),
    UserAlreadyExists(UserAlreadyExists),
}

#[derive(Union)]
pub enum RemoveUserResult{
    UserRemoved(User),
    UserNotFound(UserNotFound),
}

#[derive(Default)]
pub struct UsersMutation;

#[Object]
impl UsersMutation{
    pub async fn add_user(&self, ctx: &Context<'_>, mut user: User)->AddUserResult{
        let state=ctx.data_unchecked::<SharedChatRoomState>();
        {
            // Check if user exists
            let state=state.read().await;
            if state.users.iter().any(|u| u.name==user.name){
                return AddUserResult::UserAlreadyExists(UserAlreadyExists { user_name: user.name.clone() });
            }

            // Check if the room user is in is valid, if not set it to None
            if let Some(room_name) = &user.room {
                if !state.rooms.iter().any(|r| r.name == *room_name) {
                    user.room=None;
                }
            }
        }

        let mut state=state.write().await;
        state.users.push(user.clone());

        // Publish to event bus
        let event_bus=ctx.data_unchecked::<EventBus>();
        //TODO handle error
        let _ = event_bus.user.send(UserEvent::UserAdded(user.clone()));

        // If user has a default room, publish userentered to room event stream
        if let Some(room_name) = &user.room {
            let _ = event_bus.room.send(RoomEvent::UserEntered(room_name.to_string(), user.clone()));
        }

        AddUserResult::UserAdded(user)
    }

    pub async fn remove_user(&self, ctx:&Context<'_>, user_name: String)->RemoveUserResult{
        let state=ctx.data_unchecked::<SharedChatRoomState>();
        let mut state=state.write().await;

        // Find user position
        if let Some(pos)=state.users.iter().position(|u|u.name==user_name){
            // Remove user
            let removed_user=state.users.remove(pos);
            // If user is in a room, publish user left event
            if let Some(room_name) = &removed_user.room {
                let event_bus=ctx.data_unchecked::<EventBus>();
                let _ = event_bus.room.send(RoomEvent::UserLeft(room_name.to_string(), removed_user.clone()));
            }
            // Return removed user name
            RemoveUserResult::UserRemoved(removed_user)
        }else{
            // Return user not found
            RemoveUserResult::UserNotFound(UserNotFound{ name: user_name })
        }
    }
}

#[derive(Default)]
pub struct UsersSubscription;

#[Subscription]
impl UsersSubscription {
    pub async fn user_added(&self, ctx: &Context<'_>) -> impl Stream<Item = User> {
        let rx = ctx.data_unchecked::<EventBus>().user.subscribe();
        BroadcastStream::new(rx).filter_map(|event| async move{
            match event {
                Ok(UserEvent::UserAdded(user)) => Some(user),
                _ => None, // Return None to filter out other events or errors
            }
        })
    }

    pub async fn user_removed(&self, ctx: &Context<'_>)-> impl Stream<Item=String>{
        let rx=ctx.data_unchecked::<EventBus>().user.subscribe();
        BroadcastStream::new(rx).filter_map(|event|async move{
            match event{
                Ok(UserEvent::UserRemoved(user_name))=>Some(user_name),
                _=>None
            }
        })
    }
}
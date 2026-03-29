use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject, Subscription, Union};
use domain::events::RoomEvent;
use futures_util::{Stream, StreamExt};
use service::ServiceContainer;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use crate::types::error::{MissingIdempotencyKeyError, RoomError};
use crate::types::idempotency::IdempotencyHeader;
use crate::types::room::{CreateRoomInput, Room, TypingIndicator};

#[derive(Default)]
pub struct RoomQuery;

#[Object]
impl RoomQuery {
    /// Fetch all rooms.
    async fn rooms(&self, ctx: &Context<'_>) -> RoomListResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.room.get_all_rooms().await {
            Ok(rooms) => {
                let items = rooms.into_iter().map(Room::from).collect();
                RoomListResult::Rooms(RoomList { items })
            }
            Err(e) => RoomListResult::Error(e.into()),
        }
    }

    /// Fetch a single room by its public ID.
    async fn room(&self, ctx: &Context<'_>, id: Uuid) -> GetRoomResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.room.get_room_by_id(id).await {
            Ok(room) => GetRoomResult::Room(Room::from(room)),
            Err(e) => GetRoomResult::Error(e.into()),
        }
    }
}

#[derive(Default)]
pub struct RoomMutation;

#[Object]
impl RoomMutation {
    /// Create a new room. Requires the `Idempotency-Key` HTTP header.
    async fn create_room(&self, ctx: &Context<'_>, input: CreateRoomInput) -> CreateRoomResult {
        let idempotency_key = match ctx.data_unchecked::<IdempotencyHeader>().0 {
            Some(key) => key,
            None => {
                return CreateRoomResult::Error(RoomError::MissingIdempotencyKey(
                    MissingIdempotencyKeyError::new(),
                ));
            }
        };

        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services
            .room
            .add_room(input.name, input.capacity, idempotency_key)
            .await
        {
            Ok(room) => CreateRoomResult::Room(Room::from(room)),
            Err(e) => CreateRoomResult::Error(e.into()),
        }
    }

    //TODO update room (e.g. change name or capacity)

    /// Delete a room by its public ID.
    async fn delete_room(&self, ctx: &Context<'_>, id: Uuid) -> DeleteRoomResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.room.delete_room(id).await {
            Ok(id) => DeleteRoomResult::Success(DeletedId { id }),
            Err(e) => DeleteRoomResult::Error(e.into()),
        }
    }

    /// Set a user's typing status in a room.
    async fn set_typing(
        &self,
        ctx: &Context<'_>,
        room_id: Uuid,
        user_id: Uuid,
        is_typing: bool,
    ) -> SetTypingResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services
            .room
            .set_user_typing(room_id, user_id, is_typing)
            .await
        {
            Ok(()) => SetTypingResult::Success(TypingSuccess { ok: true }),
            Err(e) => SetTypingResult::Error(e.into()),
        }
    }
}

#[derive(Default)]
pub struct RoomSubscription;

#[Subscription]
impl RoomSubscription {
    /// Subscribe to events when a new room is added.
    async fn room_added(&self, ctx: &Context<'_>) -> impl Stream<Item = Room> {
        let rx = ctx
            .data_unchecked::<Arc<ServiceContainer>>()
            .event_bus
            .room
            .subscribe();

        // Filter the broadcast stream to only include "Added" events, and map to the Room type.
        BroadcastStream::new(rx).filter_map(|event| async move {
            match event {
                Ok(RoomEvent::Added(room)) => Some(Room::from(room)),
                Err(e) => {
                    tracing::warn!("room_added subscription error: {e}");
                    None
                }
                _ => None,
            }
        })
    }

    /// Subscribe to typing indicators for a specific room.
    async fn user_typing(
        &self,
        ctx: &Context<'_>,
        room_id: Uuid,
    ) -> impl Stream<Item = TypingIndicator> {
        let rx = ctx
            .data_unchecked::<Arc<ServiceContainer>>()
            .event_bus
            .room
            .subscribe();

        // Filter the broadcast stream to only include "UserTyping" events for the specified room, and map to TypingIndicator.
        BroadcastStream::new(rx).filter_map(move |event| async move {
            match event {
                Ok(RoomEvent::UserTyping(typing)) if typing.room_id == room_id => {
                    Some(TypingIndicator::from(typing))
                }
                Err(e) => {
                    tracing::warn!("user_typing subscription error: {e}");
                    None
                }
                _ => None,
            }
        })
    }
}

/// Wrapper so `Vec<Room>` can be a GraphQL union variant.
#[derive(SimpleObject)]
pub struct RoomList {
    pub items: Vec<Room>,
}

/// Wrapper for returning a deleted entity's ID.
#[derive(SimpleObject)]
pub struct DeletedId {
    pub id: Uuid,
}

// ---------------------------------------------------------------------------
// Result unions
// ---------------------------------------------------------------------------

#[derive(Union)]
pub enum RoomListResult {
    Rooms(RoomList),
    #[graphql(flatten)]
    Error(RoomError),
}

#[derive(Union)]
pub enum GetRoomResult {
    Room(Room),
    #[graphql(flatten)]
    Error(RoomError),
}

#[derive(Union)]
pub enum CreateRoomResult {
    Room(Room),
    #[graphql(flatten)]
    Error(RoomError),
}

#[derive(Union)]
pub enum DeleteRoomResult {
    Success(DeletedId),
    #[graphql(flatten)]
    Error(RoomError),
}

/// Wrapper for a successful typing status update.
#[derive(SimpleObject)]
pub struct TypingSuccess {
    pub ok: bool,
}

#[derive(Union)]
pub enum SetTypingResult {
    Success(TypingSuccess),
    #[graphql(flatten)]
    Error(RoomError),
}

use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject, Union};
use service::ServiceContainer;
use uuid::Uuid;

use crate::types::idempotency::IdempotencyHeader;
use crate::types::error::{MissingIdempotencyKeyError, RoomError};
use crate::types::room::{CreateRoomInput, Room};

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

    // TODO fetch rooms with available capacity (requires joining with users_in_room count)
    // TODO fetch list of users in a room
}

#[derive(Default)]
pub struct RoomMutation;

#[Object]
impl RoomMutation {
    /// Create a new room. Requires the `Idempotency-Key` HTTP header.
    async fn create_room(&self, ctx: &Context<'_>, input: CreateRoomInput) -> CreateRoomResult {
        let idempotency_key = match ctx.data_unchecked::<IdempotencyHeader>().0 {
            Some(key) => key,
            None => return CreateRoomResult::Error(RoomError::MissingIdempotencyKey(MissingIdempotencyKeyError::new())),
        };

        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.room.add_room(input.name, input.capacity, idempotency_key).await {
            Ok(room) => CreateRoomResult::Room(Room::from(room)),
            Err(e) => CreateRoomResult::Error(e.into()),
        }
    }

    /// Delete a room by its public ID.
    async fn delete_room(&self, ctx: &Context<'_>, id: Uuid) -> DeleteRoomResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.room.delete_room(id).await {
            Ok(id) => DeleteRoomResult::Success(DeletedId { id }),
            Err(e) => DeleteRoomResult::Error(e.into()),
        }
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

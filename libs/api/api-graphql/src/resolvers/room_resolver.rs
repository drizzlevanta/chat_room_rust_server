use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use service::ServiceContainer;
use uuid::Uuid;

use crate::types::room::{CreateRoomInput, Room};

#[derive(Default)]
pub struct RoomQuery;

// TODO graphql error handling

#[Object]
impl RoomQuery {
    /// Fetch all rooms.
    async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<Room>> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let rooms = services
            .room
            .get_all_rooms()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(rooms.into_iter().map(Room::from).collect())
    }

    /// Fetch a single room by its public ID.
    async fn room(&self, ctx: &Context<'_>, id: Uuid) -> Result<Room> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let room = services
            .room
            .get_room_by_id(id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Room::from(room))
    }

    // TODO fetch rooms with available capacity (requires joining with users_in_room count)
    // TODO fetch list of users in a room
}

#[derive(Default)]
pub struct RoomMutation;

#[Object]
impl RoomMutation {
    /// Create a new room.
    async fn create_room(&self, ctx: &Context<'_>, input: CreateRoomInput) -> Result<Room> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let room = services
            .room
            .add_room(input.name, input.capacity)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Room::from(room))
    }

    /// Delete a room by its public ID. Returns the deleted room's ID.
    async fn delete_room(&self, ctx: &Context<'_>, id: Uuid) -> Result<Uuid> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let deleted_id = services
            .room
            .delete_room(id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(deleted_id)
    }
}

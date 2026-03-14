use async_graphql::{InputObject, SimpleObject};
use uuid::Uuid;

/// GraphQL output type for a room.
#[derive(SimpleObject, Clone)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: u32,
}

impl From<domain::room::Room> for Room {
    fn from(r: domain::room::Room) -> Self {
        Self {
            id: r.id,
            name: r.name,
            description: r.description,
            capacity: r.capacity,
        }
    }
}

/// GraphQL input for creating a room.
#[derive(InputObject)]
pub struct CreateRoomInput {
    pub name: String,
    pub capacity: Option<u32>,
}

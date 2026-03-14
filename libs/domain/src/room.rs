use uuid::Uuid;

/// Domain representation of a room
#[derive(Clone, Debug)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: u32,
}

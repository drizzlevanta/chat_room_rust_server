use uuid::Uuid;

/// Domain representation of a room
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: i32,
}

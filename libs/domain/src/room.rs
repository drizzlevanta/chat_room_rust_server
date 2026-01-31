use uuid::Uuid;

/// Domain representation of a room
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: i32,
}

impl Room {
    pub fn new(name: String, description: Option<String>, capacity: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            capacity,
        }
    }
}

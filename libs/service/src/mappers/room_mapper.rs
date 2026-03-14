/// Mapper functions to convert between domain and entity models for Room.
use crate::mappers::EntityToDomain;
use domain::room::Room as DomainRoom;
use entity::room::Model as EntityRoom;

/// Implementation of EntityToDomain trait for Room entity
impl EntityToDomain<DomainRoom> for EntityRoom {
    fn entity_to_domain(self, _context: ()) -> DomainRoom {
        DomainRoom {
            id: self.public_id,
            name: self.name,
            capacity: self.capacity as u32,
            description: self.description,
        }
    }
}

/// Mapper functions to convert between domain and entity models for Room.
use domain::room::Room as DomainRoom;
use entity::room::Model as EntityRoom;

pub fn entity_to_domain(entity: EntityRoom) -> DomainRoom {
    DomainRoom {
        id: entity.public_id,
        name: entity.name,
        capacity: entity.capacity,
        description: entity.description,
    }
}

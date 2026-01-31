use domain::user::{ParseUserStatusError, Status, User as DomainUser};
use entity::room::Model as Room;
use entity::user::Model as EntityUser;
use uuid::Uuid;

/// Parameter to represent a Room in different forms. Either passed as an entity or just its public id.
pub enum RoomParam {
    Entity(Room),
    PublicId(Uuid),
}

impl RoomParam {
    fn into_public_id(self) -> Uuid {
        match self {
            RoomParam::Entity(room) => room.public_id,
            RoomParam::PublicId(id) => id,
        }
    }
}

/// Mapper functions to convert between domain and entity models for User.
pub fn entity_to_domain(
    entity: EntityUser,
    room: Option<RoomParam>,
) -> Result<DomainUser, ParseUserStatusError> {
    let status = entity.status.map(|s| s.parse::<Status>()).transpose()?;

    Ok(DomainUser {
        id: entity.public_id,
        name: entity.name,
        status: status,
        room: room.map(|r| r.into_public_id()),
        last_seen: entity.last_seen_at,
    })
}

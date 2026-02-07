use crate::mappers::TryEntityToDomain;
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

/// Implementation of TryEntityToDomain trait for User entity
impl TryEntityToDomain<DomainUser, Option<RoomParam>> for EntityUser {
    type Error = ParseUserStatusError;

    fn try_entity_to_domain(self, room: Option<RoomParam>) -> Result<DomainUser, Self::Error> {
        let status = self.status.map(|s| s.parse::<Status>()).transpose()?;

        Ok(DomainUser {
            id: self.public_id,
            name: self.name,
            status,
            room: room.map(|r| r.into_public_id()),
            last_seen: self.last_seen_at,
        })
    }
}

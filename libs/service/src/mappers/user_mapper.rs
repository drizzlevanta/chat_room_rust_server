use domain::user::{ParseUserStatusError, Status, User as DomainUser};
use entity::room::Model as Room;
use entity::user::Model as EntityUser;

/// Mapper functions to convert between domain and entity models for User.
pub fn entity_to_domain(
    entity: EntityUser,
    room: Option<Room>,
) -> Result<DomainUser, ParseUserStatusError> {
    let status = entity.status.map(|s| s.parse::<Status>()).transpose()?;

    Ok(DomainUser {
        id: entity.public_id,
        name: entity.name,
        status: status,
        room: room.map(|r| r.public_id),
        last_seen: entity.last_seen_at,
    })
}

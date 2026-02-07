use crate::mappers::EntityToDomain;
use domain::message::Message;
use entity::message::Model as EntityMessage;
use uuid::Uuid;

/// Context needed for converting a Message entity to domain
pub struct MessageContext {
    pub user_id: Uuid,
    pub room_id: Uuid,
}

/// Implementation of EntityToDomain trait for Message entity
impl EntityToDomain<Message, MessageContext> for EntityMessage {
    fn entity_to_domain(self, context: MessageContext) -> Message {
        Message {
            id: self.public_id,
            created_at: self.created_at,
            content: self.content,
            sender: context.user_id,
            room: context.room_id,
        }
    }
}

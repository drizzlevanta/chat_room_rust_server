use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject, Union};
use service::ServiceContainer;
use uuid::Uuid;

use crate::types::error::{MessageError, MissingIdempotencyKeyError};
use crate::types::idempotency::IdempotencyHeader;
use crate::types::message::{Message, SendMessageInput};
use crate::types::pagination::MessagePage;

#[derive(Default)]
pub struct MessageQuery;

#[Object]
impl MessageQuery {
    /// Fetch all messages in a room (use with caution for large histories).
    async fn messages(&self, ctx: &Context<'_>, room_id: Uuid) -> MessageListResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.message.get_all_messages_in_room(room_id).await {
            Ok(messages) => {
                let items = messages.into_iter().map(Message::from).collect();
                MessageListResult::Messages(MessageList { items })
            }
            Err(e) => MessageListResult::Error(e.into()),
        }
    }

    /// Fetch paginated messages in a room (cursor-based, newest first).
    ///
    /// Pass `None` for `cursor` to start from the most recent messages.
    async fn messages_paginated(
        &self,
        ctx: &Context<'_>,
        room_id: Uuid,
        cursor: Option<Uuid>,
        #[graphql(default = 20)] limit: u64,
    ) -> MessagePageResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services
            .message
            .get_messages_in_room(room_id, cursor, limit)
            .await
        {
            Ok(page) => MessagePageResult::Page(MessagePage::from(page)),
            Err(e) => MessagePageResult::Error(e.into()),
        }
    }
}

#[derive(Default)]
pub struct MessageMutation;

#[Object]
impl MessageMutation {
    /// Send a message to a room. Requires the `Idempotency-Key` HTTP header.
    async fn send_message(&self, ctx: &Context<'_>, input: SendMessageInput) -> SendMessageResult {
        let idempotency_key = match ctx.data_unchecked::<IdempotencyHeader>().0 {
            Some(key) => key,
            None => {
                return SendMessageResult::Error(MessageError::MissingIdempotencyKey(
                    MissingIdempotencyKeyError::new(),
                ));
            }
        };

        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services
            .message
            .add_message(&input.content, input.sender, input.room, idempotency_key)
            .await
        {
            Ok(msg) => SendMessageResult::Message(Message::from(msg)),
            Err(e) => SendMessageResult::Error(e.into()),
        }
    }

    /// Delete a message by its public ID.
    async fn delete_message(&self, ctx: &Context<'_>, id: Uuid) -> DeleteMessageResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.message.delete_message(id).await {
            Ok(message_id) => DeleteMessageResult::Success(DeleteSuccess { id: message_id }),
            Err(e) => DeleteMessageResult::Error(e.into()),
        }
    }
}

/// Wrapper so `Vec<Message>` can be a GraphQL union variant.
#[derive(SimpleObject)]
pub struct MessageList {
    pub items: Vec<Message>,
}

/// Wrapper for successful delete operations.
#[derive(SimpleObject)]
pub struct DeleteSuccess {
    pub id: Uuid,
}

/// Result union for queries returning a list of messages.
#[derive(Union)]
pub enum MessageListResult {
    Messages(MessageList),

    // GraphQL does not allow nested unions, so we flatten the error
    #[graphql(flatten)]
    Error(MessageError),
}

/// Result union for paginated message queries.
#[derive(Union)]
pub enum MessagePageResult {
    Page(MessagePage),
    #[graphql(flatten)]
    Error(MessageError),
}

/// Result union for the send-message mutation.
#[derive(Union)]
pub enum SendMessageResult {
    Message(Message),
    #[graphql(flatten)]
    Error(MessageError),
}

/// Result union for the delete-message mutation.
#[derive(Union)]
pub enum DeleteMessageResult {
    Success(DeleteSuccess),
    #[graphql(flatten)]
    Error(MessageError),
}

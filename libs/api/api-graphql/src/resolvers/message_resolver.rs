use std::sync::Arc;

use async_graphql::{Context, Object, Result, SimpleObject, Union};
use service::ServiceContainer;
use service::message_service::MessageServiceError;
use uuid::Uuid;

use crate::types::message::{Message, SendMessageInput};
use crate::types::pagination::MessagePage;

#[derive(Default)]
pub struct MessageQuery;

#[Object]
impl MessageQuery {
    /// Fetch all messages in a room (use with caution for large histories).
    /// TODO break down MessageQueryResult into separate queries for list vs paginated to avoid the union overhead when not needed.
    async fn messages(&self, ctx: &Context<'_>, room_id: Uuid) -> MessageQueryResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.message.get_all_messages_in_room(room_id).await {
            // Can't use `?` here because we want to return a GraphQL union with either the messages or an error, not just propagate the error.
            Ok(messages) => {
                let items = messages.into_iter().map(Message::from).collect();
                MessageQueryResult::Messages(MessageList { items })
            }
            Err(e) => MessageQueryResult::Error(MessageError::from(e)),
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
    ) -> Result<MessagePage> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let page = services
            .message
            .get_messages_in_room(room_id, cursor, limit)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(MessagePage::from(page))
    }

    /// Fetch the latest N messages in a room (newest first).
    async fn latest_messages(
        &self,
        ctx: &Context<'_>,
        room_id: Uuid,
        #[graphql(default = 10)] n: usize,
    ) -> Result<Vec<Message>> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let messages = services
            .message
            .get_latest_n_messages_in_room(room_id, n)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(messages.into_iter().map(Message::from).collect())
    }
}

#[derive(Default)]
pub struct MessageMutation;

#[Object]
impl MessageMutation {
    /// Send a message to a room. Uses idempotency key to prevent duplicates.
    async fn send_message(&self, ctx: &Context<'_>, input: SendMessageInput) -> Result<Message> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let msg = services
            .message
            .add_message(
                &input.content,
                input.sender,
                input.room,
                input.idempotency_key,
            )
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Message::from(msg))
    }

    /// Delete a message by its public ID.
    async fn delete_message(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        services
            .message
            .delete_message(id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }
}

/// Message-related errors exposed to the GraphQL client.
// Conceals database error details from the client for security.
#[derive(SimpleObject, Debug)]
pub struct MessageError {
    pub message: String,
    pub code: String,
}

impl From<MessageServiceError> for MessageError {
    fn from(err: MessageServiceError) -> Self {
        let msg = err.to_string();
        match err {
            MessageServiceError::MessageTooLong => Self {
                message: msg,
                code: "MESSAGE_TOO_LONG".to_string(),
            },
            MessageServiceError::UserNotFoundinRoom { .. } => Self {
                message: msg,
                code: "USER_NOT_IN_ROOM".to_string(),
            },
            MessageServiceError::RoomNotFound(_) => Self {
                message: msg,
                code: "ROOM_NOT_FOUND".to_string(),
            },
            MessageServiceError::MessageNotFound(_) => Self {
                message: msg,
                code: "MESSAGE_NOT_FOUND".to_string(),
            },
            // Conceal the database error details from the client for security
            MessageServiceError::DatabaseError(e) => {
                //TODO log this properly instead of just printing to stderr
                eprintln!("Database error: {e}");
                Self {
                    message: "Internal server error".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }
            }
        }
    }
}

/// Wrapper so `Vec<Message>` can be a GraphQL union variant.
#[derive(SimpleObject)]
pub struct MessageList {
    pub items: Vec<Message>,
}

/// GraphQL union type for message query results.
#[derive(Union)]
pub enum MessageQueryResult {
    Messages(MessageList), //TODO check if we can just return Vec<Message> directly without wrapping in MessageList
    MessagesPaginated(MessagePage),
    Error(MessageError),
}

/// GraphQL union type for message mutation results.
#[derive(Union)]
pub enum MessageMutationResult {
    Message(Message),
    Error(MessageError),
}

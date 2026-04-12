use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject, Subscription, Union};
use domain::events::MessageEvent;
use futures_util::{Stream, StreamExt};
use service::ServiceContainer;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::types::error::{MessageError, MissingIdempotencyKeyError};
use crate::types::idempotency::IdempotencyHeader;
use crate::types::message::{EditMessageInput, Message, MessageEditedEvent, MessageSentEvent, SendMessageInput};
use crate::types::pagination::MessagePage;

#[derive(Default)]
pub struct MessageQuery;

#[Object]
impl MessageQuery {
    /// Fetch all messages in a room (use with caution for large histories).
    #[instrument(skip(self, ctx), name = "Fetch all messages in room")]
    async fn messages(&self, ctx: &Context<'_>, room_id: Uuid) -> MessageListResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.message.get_all_messages_in_room(room_id).await {
            Ok(messages) => {
                info!("Fetched {} messages", messages.len());
                let items = messages.into_iter().map(Message::from).collect();
                MessageListResult::Messages(MessageList { items })
            }
            Err(e) => MessageListResult::Error(e.into()),
        }
    }

    /// Fetch paginated messages in a room (cursor-based, newest first).
    ///
    /// Pass `None` for `cursor` to start from the most recent messages.
    #[instrument(skip(self, ctx), name = "Fetch paginated messages in room")]
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
            Ok(page) => {
                info!("Fetched paginated messages");
                MessagePageResult::Page(MessagePage::from(page))
            }
            Err(e) => MessagePageResult::Error(e.into()),
        }
    }
}

#[derive(Default)]
pub struct MessageMutation;

#[Object]
impl MessageMutation {
    /// Send a message to a room. Requires the `Idempotency-Key` HTTP header.
    #[instrument(skip(self, ctx), name = "Send message")]
    async fn send_message(&self, ctx: &Context<'_>, input: SendMessageInput) -> SendMessageResult {
        let idempotency_key = match ctx.data_unchecked::<IdempotencyHeader>().0 {
            Some(key) => key,
            None => {
                error!("Missing Idempotency-Key header");
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
            Ok(msg) => {
                info!("Sent message");
                SendMessageResult::Message(Message::from(msg))
            }
            Err(e) => SendMessageResult::Error(e.into()),
        }
    }

    /// Edit the content of an existing message.
    async fn edit_message(&self, ctx: &Context<'_>, input: EditMessageInput) -> EditMessageResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.message.edit_message(input.id, &input.content).await {
            Ok(msg) => EditMessageResult::Message(Message::from(msg)),
            Err(e) => EditMessageResult::Error(e.into()),
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

#[derive(Default)]
pub struct MessageSubscription;

#[Subscription]
impl MessageSubscription {
    async fn message_sent(
        &self,
        ctx: &Context<'_>,
        room_id: Uuid,
    ) -> impl Stream<Item = MessageSentEvent> {
        let rx = ctx
            .data_unchecked::<Arc<ServiceContainer>>()
            .event_bus
            .message
            .subscribe();

        BroadcastStream::new(rx).filter_map(move |event| async move {
            match event {
                Ok(MessageEvent::Sent(msg)) if msg.room == room_id => {
                    Some(MessageSentEvent::from(msg))
                }
                Ok(_) => None,
                Err(BroadcastStreamRecvError::Lagged(n)) => {
                    warn!("message_sent subscription lagged, dropped {} messages", n);
                    None
                }
            }
        })
    }

    /// Subscribe to message edits in a room.
    async fn message_edited(
        &self,
        ctx: &Context<'_>,
        room_id: Uuid,
    ) -> impl Stream<Item = MessageEditedEvent> {
        let rx = ctx
            .data_unchecked::<Arc<ServiceContainer>>()
            .event_bus
            .message
            .subscribe();

        BroadcastStream::new(rx).filter_map(move |event| async move {
            match event {
                Ok(MessageEvent::Edited(msg)) if msg.room == room_id => {
                    Some(MessageEditedEvent::from(msg))
                }
                Ok(_) => None,
                Err(BroadcastStreamRecvError::Lagged(n)) => {
                    warn!("message_edited subscription lagged, dropped {} messages", n);
                    None
                }
            }
        })
    }

    /// Subscribe to message deletions in a room. Yields the deleted message's public ID.
    async fn message_deleted(&self, ctx: &Context<'_>, room_id: Uuid) -> impl Stream<Item = Uuid> {
        let rx = ctx
            .data_unchecked::<Arc<ServiceContainer>>()
            .event_bus
            .message
            .subscribe();

        BroadcastStream::new(rx).filter_map(move |event| async move {
            match event {
                Ok(MessageEvent::Deleted {
                    message_id,
                    room_id: event_room_id,
                }) if event_room_id == room_id => Some(message_id),
                Ok(_) => None,
                Err(BroadcastStreamRecvError::Lagged(n)) => {
                    warn!("message_deleted subscription lagged, dropped {} messages", n);
                    None
                }
            }
        })
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

/// Result union for the edit-message mutation.
#[derive(Union)]
pub enum EditMessageResult {
    Message(Message),
    #[graphql(flatten)]
    Error(MessageError),
}

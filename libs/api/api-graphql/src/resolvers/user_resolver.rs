use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject, Subscription, Union};
use domain::events::UserEvent;
use futures_util::{Stream, StreamExt};
use service::ServiceContainer;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tracing::warn;
use uuid::Uuid;

use crate::types::error::{MissingIdempotencyKeyError, UserError};
use crate::types::idempotency::IdempotencyHeader;
use crate::types::user::{
    CreateUserInput, UpdateUserStatusInput, User, UserStatus, UserStatusChanged,
};

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// Fetch a single user by public ID.
    async fn user(&self, ctx: &Context<'_>, id: Uuid) -> GetUserResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.user.get_user_by_id(id).await {
            Ok(user) => GetUserResult::User(User::from(user)),
            Err(e) => GetUserResult::Error(e.into()),
        }
    }

    /// Fetch all users currently in a specific room.
    async fn users_in_room(&self, ctx: &Context<'_>, room_id: Uuid) -> UserListResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.user.get_users_in_room(room_id).await {
            Ok(users) => {
                let items = users.into_iter().map(User::from).collect();
                UserListResult::Users(UserList { items })
            }
            Err(e) => UserListResult::Error(e.into()),
        }
    }

    /// Fetch all users with a given status.
    async fn users_by_status(&self, ctx: &Context<'_>, status: UserStatus) -> UserListResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.user.get_users_by_status(status.into(), None).await {
            Ok(users) => {
                let items = users.into_iter().map(User::from).collect();
                UserListResult::Users(UserList { items })
            }
            Err(e) => UserListResult::Error(e.into()),
        }
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    /// Create a new user, optionally placing them in a room.
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> CreateUserResult {
        // Try to get the idempotency key from the HTTP header. If it's missing, return an error variant immediately.
        let idempotency_key = match ctx.data_unchecked::<IdempotencyHeader>().0 {
            Some(key) => key,
            None => {
                return CreateUserResult::Error(UserError::MissingIdempotencyKey(
                    MissingIdempotencyKeyError::new(),
                ));
            }
        };

        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services
            .user
            .add_user(input.name, input.room, idempotency_key)
            .await
        {
            Ok(user) => CreateUserResult::User(User::from(user)),
            Err(e) => CreateUserResult::Error(e.into()),
        }
    }

    /// Update a user's status (Online, Offline, Away).
    async fn update_user_status(
        &self,
        ctx: &Context<'_>,
        input: UpdateUserStatusInput,
    ) -> UpdateUserStatusResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services
            .user
            .update_user_status(input.user_id, input.status.into())
            .await
        {
            Ok(()) => UpdateUserStatusResult::Success(UpdateSuccess { success: true }),
            Err(e) => UpdateUserStatusResult::Error(e.into()),
        }
    }

    /// Delete a user by public ID.
    async fn delete_user(&self, ctx: &Context<'_>, id: Uuid) -> DeleteUserResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.user.delete_user(id).await {
            Ok(id) => DeleteUserResult::Success(DeletedUserId { id }),
            Err(e) => DeleteUserResult::Error(e.into()),
        }
    }

    /// Set user's presence in a room (e.g. when they join or leave).
    async fn set_presence(
        &self,
        ctx: &Context<'_>,
        room_id: Uuid,
        user_id: Uuid,
        is_present: bool,
    ) -> SetPresenceResult {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let result = if is_present {
            services.user.join_room(user_id, room_id).await
        } else {
            services.user.leave_room(user_id, room_id).await
        };
        match result {
            Ok(()) => SetPresenceResult::Success(UpdateSuccess { success: true }),
            Err(e) => SetPresenceResult::Error(e.into()),
        }
    }
}

#[derive(Default)]
pub struct UserSubscription;

#[Subscription]
impl UserSubscription {
    /// Subscribe to user status changes across all users.
    async fn user_status_changed(
        &self,
        ctx: &Context<'_>,
    ) -> impl Stream<Item = UserStatusChanged> {
        let rx = ctx
            .data_unchecked::<Arc<ServiceContainer>>()
            .event_bus
            .user
            .subscribe();

        BroadcastStream::new(rx).filter_map(|event| async move {
            match event {
                Ok(UserEvent::StatusChanged { user_id, status }) => Some(UserStatusChanged {
                    user_id,
                    status: status.into(),
                }),
                Ok(_) => None,
                Err(BroadcastStreamRecvError::Lagged(n)) => {
                    warn!(
                        "user_status_changed subscription lagged, dropped {} events",
                        n
                    );
                    None
                }
            }
        })
    }
}

/// Wrapper so `Vec<User>` can be a GraphQL union variant.
#[derive(SimpleObject)]
pub struct UserList {
    pub items: Vec<User>,
}

/// Wrapper for successful status update.
#[derive(SimpleObject)]
pub struct UpdateSuccess {
    pub success: bool,
}

/// Wrapper for returning a deleted user's ID.
#[derive(SimpleObject)]
pub struct DeletedUserId {
    pub id: Uuid,
}

// ---------------------------------------------------------------------------
// Result unions
// ---------------------------------------------------------------------------

#[derive(Union)]
pub enum UserListResult {
    Users(UserList),
    #[graphql(flatten)]
    Error(UserError),
}

#[derive(Union)]
pub enum GetUserResult {
    User(User),
    #[graphql(flatten)]
    Error(UserError),
}

#[derive(Union)]
pub enum CreateUserResult {
    User(User),
    #[graphql(flatten)]
    Error(UserError),
}

#[derive(Union)]
pub enum UpdateUserStatusResult {
    Success(UpdateSuccess),
    #[graphql(flatten)]
    Error(UserError),
}

#[derive(Union)]
pub enum DeleteUserResult {
    Success(DeletedUserId),
    #[graphql(flatten)]
    Error(UserError),
}

#[derive(Union)]
pub enum SetPresenceResult {
    Success(UpdateSuccess),
    #[graphql(flatten)]
    Error(UserError),
}

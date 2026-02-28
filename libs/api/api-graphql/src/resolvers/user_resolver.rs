use std::sync::Arc;

use async_graphql::{Context, Object, SimpleObject, Union};
use service::ServiceContainer;
use uuid::Uuid;

use crate::types::error::UserError;
use crate::types::user::{CreateUserInput, UpdateUserStatusInput, User, UserStatus};

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
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        match services.user.add_user(input.name, input.room).await {
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

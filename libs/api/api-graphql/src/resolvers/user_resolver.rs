use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use service::ServiceContainer;
use uuid::Uuid;

use crate::types::user::{CreateUserInput, UpdateUserStatusInput, User, UserStatus};

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// Fetch a single user by public ID.
    async fn user(&self, ctx: &Context<'_>, id: Uuid) -> Result<User> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let user = services
            .user
            .get_user_by_id(id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(User::from(user))
    }

    /// Fetch all users currently in a specific room.
    async fn users_in_room(&self, ctx: &Context<'_>, room_id: Uuid) -> Result<Vec<User>> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let users = services
            .user
            .get_users_in_room(room_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(users.into_iter().map(User::from).collect())
    }

    /// Fetch all users with a given status.
    async fn users_by_status(&self, ctx: &Context<'_>, status: UserStatus) -> Result<Vec<User>> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let users = services
            .user
            .get_users_by_status(status.into())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(users.into_iter().map(User::from).collect())
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    /// Create a new user, optionally placing them in a room.
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<User> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let user = services
            .user
            .add_user(input.name, input.room)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(User::from(user))
    }

    /// Update a user's status (Online, Offline, Away).
    async fn update_user_status(
        &self,
        ctx: &Context<'_>,
        input: UpdateUserStatusInput,
    ) -> Result<bool> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        services
            .user
            .update_user_status(input.user_id, input.status.into())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Delete a user by public ID. Returns the deleted user's ID.
    async fn delete_user(&self, ctx: &Context<'_>, id: Uuid) -> Result<Uuid> {
        let services = ctx.data_unchecked::<Arc<ServiceContainer>>();
        let deleted_id = services
            .user
            .delete_user(id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(deleted_id)
    }
}

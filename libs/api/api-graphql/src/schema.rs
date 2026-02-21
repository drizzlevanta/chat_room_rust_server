use std::sync::Arc;

use async_graphql::{MergedObject, Schema};
use service::ServiceContainer;

use crate::resolvers::message_resolver::{MessageMutation, MessageQuery};
use crate::resolvers::room_resolver::{RoomMutation, RoomQuery};
use crate::resolvers::user_resolver::{UserMutation, UserQuery};

/// Root query type merging all domain queries.
#[derive(MergedObject, Default)]
pub struct QueryRoot(RoomQuery, UserQuery, MessageQuery);

/// Root mutation type merging all domain mutations.
#[derive(MergedObject, Default)]
pub struct MutationRoot(RoomMutation, UserMutation, MessageMutation);

/// The full GraphQL schema type exposed by this crate.
pub type AppSchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;

/// Build the GraphQL schema with the service container injected as context data.
pub fn build_schema(services: Arc<ServiceContainer>) -> AppSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        async_graphql::EmptySubscription,
    )
    .data(services)
    .finish()
}

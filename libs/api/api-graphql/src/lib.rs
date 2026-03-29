pub mod handler;
pub mod resolvers;
pub mod schema;
pub mod types;

use std::sync::Arc;

use async_graphql_axum::GraphQLSubscription;
use axum::{Router, routing::get};
use service::ServiceContainer;

use crate::handler::{graphql_handler, graphql_playground};
use crate::schema::{AppSchema, build_schema};

/// Build an Axum `Router` with GraphQL playground + query endpoint.
///
/// Accepts a shared `ServiceContainer` so the same instance can be
/// reused by other API layers (e.g. REST).
///
/// Mount this under a prefix in the top-level server, e.g.:
/// ```ignore
/// let services = ServiceContainer::new_shared(db);
/// let app = Router::new()
///     .nest("/graphql", api_graphql::graphql_router(services.clone()))
///     .nest("/api", api_rest::rest_router(services));
/// ```
pub fn graphql_router(services: Arc<ServiceContainer>) -> Router {
    let schema: AppSchema = build_schema(services);

    Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
        .with_state(schema)
}

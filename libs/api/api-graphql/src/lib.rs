pub mod handler;
pub mod resolvers;
pub mod schema;
pub mod types;

use std::sync::Arc;

use axum::{routing::get, Router};
use service::ServiceContainer;

use crate::handler::{graphql_handler, graphql_playground};
use crate::schema::{build_schema, AppSchema};

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
        .with_state(schema)
}

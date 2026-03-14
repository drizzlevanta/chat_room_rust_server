use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use uuid::Uuid;

use crate::schema::AppSchema;
use crate::types::idempotency::{IdempotencyHeader, IDEMPOTENCY_KEY_HEADER};

/// Serves the GraphQL Playground IDE.
pub async fn graphql_playground() -> axum::response::Html<String> {
    axum::response::Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql/ws"),
    ))
}

/// Handles incoming GraphQL requests.
///
/// Extracts the optional `Idempotency-Key` header and injects it into the
/// GraphQL request data so resolvers can access it via `ctx.data::<IdempotencyHeader>()`.
pub async fn graphql_handler(
    schema: State<AppSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let idempotency_key = headers
        .get(IDEMPOTENCY_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    let request = req.into_inner().data(IdempotencyHeader(idempotency_key));
    schema.execute(request).await.into()
}

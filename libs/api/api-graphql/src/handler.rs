use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;

use crate::schema::AppSchema;

/// Serves the GraphQL Playground IDE.
pub async fn graphql_playground() -> axum::response::Html<String> {
    axum::response::Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql/ws"),
    ))
}

/// Handles incoming GraphQL requests.
pub async fn graphql_handler(
    schema: State<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

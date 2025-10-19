use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;

use crate::schema::AppSchema;

// Handler for GraphQL playground
pub async fn graphql_playground() -> axum::response::Html<String> {
    axum::response::Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}


// Handler for GraphQL requests
pub async fn graphql_handler(
    schema: State<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
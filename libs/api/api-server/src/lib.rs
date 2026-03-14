use std::env;

use axum::Router;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use service::ServiceContainer;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Start the Axum server that composes all API layers (GraphQL, REST, etc.).
///
/// This is the single entry-point that wires up the database, runs
/// migrations, builds the shared `ServiceContainer`, and mounts every API sub-router
#[tokio::main]
pub async fn start_server() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // ── Logging ───────────────────────────────────────────────────────
    // Initialize tracing subscriber with environment filter (default to INFO level)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // ── Database ──────────────────────────────────────────────────────
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("Connecting to database at {db_url}");

    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    Migrator::up(&db, None).await.unwrap();
    info!("Migrations applied");

    // ── Services ─────────────────────────────────────────────────────
    // Instantiate the shared service container that will be passed to all API layers.
    let services = ServiceContainer::new_shared(db);

    // ── Router ───────────────────────────────────────────────────────
    // GraphQL endpoint
    let app = Router::new().nest("/graphql", api_graphql::graphql_router(services.clone()));
    // REST endpoint
    // .nest("/api", api_rest::rest_router(services));  // Uncomment when api-rest exposes a router

    // ── Listener ─────────────────────────────────────────────────────
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "7000".to_string());
    let addr = format!("{host}:{port}");

    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Server running on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}

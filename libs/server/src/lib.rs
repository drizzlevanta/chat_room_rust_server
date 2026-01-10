use std::env;

use async_graphql_axum::GraphQLSubscription;
use axum::{routing::get, Router};
use chat_room::shared::event_bus::EventBus;
use sea_orm::Database;
use tokio::net::TcpListener;
use migration::{Migrator, MigratorTrait};

use crate::handler::{graphql_handler, graphql_playground};

mod schema;
mod handler;

#[tokio::main]
pub async fn start_server(){
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    //TODO add DB connection here
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    println!("Connecting to database at {}", db_url);
    let conn=Database::connect(&db_url).await.expect("Failed to connect to database");
     Migrator::up(&conn, None).await.unwrap();
    println!("Migration completed at {}", db_url);
    //TODO seeding data

    let event_bus = EventBus::new(1000); // Initialize EventBus
    let schema = schema::create_schema(event_bus);

    // Set up the Axum router
    // let app = Router::new().route("/", get(graphql_playground).post(graphql_handler)).with_state(schema);
    let app = Router::new().route("/", get(graphql_playground).post(graphql_handler)).route_service("/ws", GraphQLSubscription::new(schema.clone())).with_state(schema);

    // Set up listener
    let listener = TcpListener::bind("127.0.0.1:7000").await.unwrap();
    println!("GraphQLServer running on http://127.0.0.1:7000");

    // Start the server
    axum::serve(listener, app).await.unwrap();

}
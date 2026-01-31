use dotenvy::dotenv;
use sea_orm::{Database, DbErr};
use std::env;

#[async_std::main]
async fn main() -> Result<(), DbErr> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let db_conn = Database::connect(&database_url).await?;

    seeder::seed_all(&db_conn).await?;

    println!("🌱 Database seeded successfully");
    Ok(())
}

mod m20251014_151108_create_room_table;
mod m20251017_202528_create_user_table;
mod m20251020_150040_create_message_table;


pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20251014_151108_create_room_table::Migration), Box::new(m20251017_202528_create_user_table::Migration), Box::new(m20251020_150040_create_message_table::Migration)]
    }
}

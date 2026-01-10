pub use sea_orm_migration::prelude::*;

mod message_seeder;
mod room_seeder;
mod user_seeder;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // vec![Box::new(room_seeder::Migration)]
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![
            Box::new(room_seeder::Migration),
            Box::new(user_seeder::Migration),
            Box::new(message_seeder::Migration),
        ];

        migrations.extend(migration::Migrator::migrations());
        migrations
    }
}

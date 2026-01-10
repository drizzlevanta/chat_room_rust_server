use entity::room;
use sea_orm::ActiveModelTrait;
use sea_orm_migration::{prelude::*, sea_orm::ActiveValue::Set};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        room::ActiveModel {
            name: Set("General".to_owned()),
            description: Set(Some("General discussion room".to_owned())),
            capacity: Set(100),
            public_id: Set(Uuid::new_v4()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        room::ActiveModel {
            name: Set("Tech".to_owned()),
            description: Set(Some("Technology discussion room".to_owned())),
            capacity: Set(50),
            public_id: Set(Uuid::new_v4()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        println!("Seeded initial message data.");

        Ok(())
    }
}

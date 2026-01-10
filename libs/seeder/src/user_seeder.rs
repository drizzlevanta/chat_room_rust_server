use entity::user;
use sea_orm::ActiveModelTrait;
use sea_orm_migration::{prelude::*, sea_orm::ActiveValue::Set};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        user::ActiveModel {
            name: Set("seeder_alice".to_owned()),
            status: Set(Some("online".to_owned())),
            room: Set(None),
            public_id: Set(Uuid::new_v4()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        user::ActiveModel {
            name: Set("seeder_bob".to_owned()),
            status: Set(Some("offline".to_owned())),
            room: Set(None),
            public_id: Set(Uuid::new_v4()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(())
    }
}

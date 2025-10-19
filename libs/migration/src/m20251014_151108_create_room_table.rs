use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(Room::Table).if_not_exists().col(pk_auto(Room::Id)).col(string(Room::Name)).col(string(Room::Description).null()).col(integer(Room::Capacity)).to_owned()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager.drop_table(Table::drop().table(Room::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Room {
    Table,
    Id,
    Name,
    Description, //Optional
    Capacity,
}

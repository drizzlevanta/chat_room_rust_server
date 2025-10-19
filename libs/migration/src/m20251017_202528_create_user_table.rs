use sea_orm_migration::{prelude::*, schema::*, sea_orm::{EnumIter, Iterable}};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.create_table(
            Table::create()
                .table(User::Table)
                .if_not_exists()
                .col(pk_auto(User::Id))
                .col(string(User::Name))
                .col(enumeration_null(User::Status, "status", Status::iter()))
                .col(string(User::Room)) //TODO foreign key to Room table
                .to_owned()
        )
        .await


    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
        .drop_table(Table::drop().table(User::Table).to_owned())
        .await

        //TODO drop the enum type
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Status,
    Room,
}

#[derive(Iden, EnumIter)]
pub enum Status {
    #[iden = "online"]
    Online,
    #[iden = "offline"]
    Offline,
    #[iden = "away"]
    Away,
}


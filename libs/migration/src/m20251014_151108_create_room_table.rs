use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Room::Table)
                    .if_not_exists()
                    .col(pk_auto(Room::Id))
                    .col(string(Room::Name))
                    .col(string(Room::Description).null())
                    .col(integer(Room::Capacity).default(100))
                    .col(uuid(Room::PublicId))
                    .col(timestamp(Room::CreatedAt).default(Expr::cust("CURRENT_TIMESTAMP")))
                    .col(timestamp(Room::UpdatedAt).default(Expr::cust("CURRENT_TIMESTAMP")))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Room::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Room {
    Table,
    Id, // Internal Id
    Name,
    Description, //Optional
    Capacity,
    PublicId, // Public Id exposed from API
    CreatedAt,
    UpdatedAt,
}

use sea_orm_migration::{
    prelude::*,
    schema::*,
    sea_orm::{EnumIter, Iterable},
};

use crate::m20251014_151108_create_room_table::Room;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(string(User::Name))
                    .col(enumeration_null(User::Status, "status", Status::iter()))
                    .col(integer(User::Room).null()) // nullable foreign key to Room::Id
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_room")
                            .from(User::Table, User::Room)
                            .to(Room::Table, Room::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(uuid(User::PublicId))
                    .col(timestamp(User::CreatedAt))
                    .col(timestamp(User::UpdatedAt))
                    .col(timestamp(User::LastSeenAt).null())
                    .to_owned(),
            )
            .await?;

        // Create unique index on public_id
        manager
            .create_index(
                Index::create()
                    .name("idx_user_public_id")
                    .table(User::Table)
                    .col(User::PublicId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id, // Internal Id
    Name,
    Status,
    Room,
    PublicId, // Public Id exposed from API
    CreatedAt,
    UpdatedAt,
    LastSeenAt,
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

//TODO redo enum

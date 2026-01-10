use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20251014_151108_create_room_table::Room, m20251017_202528_create_user_table::User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .if_not_exists()
                    .col(pk_auto(Message::Id))
                    .col(timestamp(Message::CreatedAt))
                    .col(string(Message::Content))
                    .col(integer(Message::Sender))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_sender")
                            .from(Message::Table, Message::Sender)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    ) // delete messages if user is deleted
                    .col(integer(Message::Room))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_room")
                            .from(Message::Table, Message::Room)
                            .to(Room::Table, Room::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    ) // delete messages if room is deleted
                    .col(uuid(Message::PublicId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Message::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Message {
    Table,
    Id,
    CreatedAt,
    Content,
    Sender,
    Room,
    PublicId,
}

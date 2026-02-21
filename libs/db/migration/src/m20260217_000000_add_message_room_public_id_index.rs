use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Composite index on (room, public_id DESC) for efficient cursor pagination.
        // Since public_id is UUIDv7 (time-ordered), ordering by it is equivalent
        // to ordering by creation time, and the composite index allows the DB to
        // satisfy both the WHERE (room) and ORDER BY (public_id DESC) from a single
        // index scan with no sorting needed.
        manager
            .create_index(
                Index::create()
                    .name("idx_message_room_public_id")
                    .table(Message::Table)
                    .col(Message::Room)
                    .col((Message::PublicId, IndexOrder::Desc))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_message_room_public_id")
                    .table(Message::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Message {
    Table,
    Room,
    PublicId,
}

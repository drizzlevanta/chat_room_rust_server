use entity::{room, user};
use sea_orm::{ActiveModelTrait, EntityTrait};
use sea_orm_migration::{prelude::*, sea_orm::ActiveValue::Set};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Get seeded users
        let users: Vec<user::Model> = user::Entity::find().all(db).await?;

        if users.len() < 2 {
            println!("Not enough users. Please seed users before seeding messages.");
            return Ok(());
        }

        // Get seeded rooms
        let rooms: Vec<room::Model> = room::Entity::find().all(db).await?;
        if rooms.len() < 2 {
            println!("Not enough rooms. Please seed rooms before seeding messages.");
            return Ok(());
        }

        // Seed messages
        for i in 0..2 {
            let message_content = format!("Hello from seeder message {}", i + 1);
            let sender = &users[i];
            let room = &rooms[i];

            entity::message::ActiveModel {
                content: Set(message_content),
                sender: Set(sender.id),
                room: Set(room.id),
                public_id: Set(Uuid::new_v4()),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        println!("Seeded initial message data.");
        let messages = entity::message::Entity::find().all(db).await?;
        for message in messages {
            println!(
                "Message ID: {}, Content: {}, Sender ID: {}, Room ID: {}",
                message.id, message.content, message.sender, message.room
            );
        }

        Ok(())
    }
}

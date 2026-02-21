use entity::{message, room, user};
use sea_orm::{ActiveModelTrait, DatabaseTransaction, DbErr, EntityTrait, Set};
use uuid::Uuid;

/// Seeds initial messages into the database.
pub async fn seed_messages(
    txn: &DatabaseTransaction,
    users: &[user::Model],
    rooms: &[room::Model],
) -> Result<(), DbErr> {
    if users.len() < 2 || rooms.len() < 2 {
        println!("Not enough rooms or users. Please seed rooms and users before seeding messages.");
        return Ok(());
    }

    // Seed messages
    for i in 0..2 {
        let message_content = format!("Hello from seeder message {}", i + 1);
        let sender = &users[i];
        let room = &rooms[i];

        message::ActiveModel {
            content: Set(message_content),
            sender: Set(sender.id),
            room: Set(room.id),
            public_id: Set(Uuid::now_v7()),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }

    println!("Seeded initial message data.");

    let messages = entity::message::Entity::find().all(txn).await?;
    for message in messages {
        println!(
            "Using new seeder! Message ID: {}, Content: {}, Sender ID: {}, Room ID: {}",
            message.id, message.content, message.sender, message.room
        );
    }

    let users = entity::user::Entity::find().all(txn).await?;
    for user in users {
        println!(
            "Using new seeder! User ID: {}, Username: {}, created_at: {}, updated_at: {}",
            user.id, user.name, user.created_at, user.updated_at
        );
    }

    let rooms = entity::room::Entity::find().all(txn).await?;
    for room in rooms {
        println!(
            "Using new seeder! Room ID: {}, Name: {}, created_at: {}, updated_at: {}",
            room.id, room.name, room.created_at, room.updated_at
        );
    }

    Ok(())
}

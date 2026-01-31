use entity::room;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseTransaction, DbErr};
use uuid::Uuid;

/// Seeds initial rooms into the database.
pub async fn seed_rooms(txn: &DatabaseTransaction) -> Result<Vec<room::Model>, DbErr> {
    let mut rooms = Vec::new();

    let room1 = room::ActiveModel {
        name: Set("General".to_owned()),
        description: Set(Some("General discussion room".to_owned())),
        capacity: Set(100),
        public_id: Set(Uuid::new_v4()),
        ..Default::default()
    }
    .insert(txn)
    .await?;

    let room2 = room::ActiveModel {
        name: Set("Tech".to_owned()),
        description: Set(Some("Technology discussion room".to_owned())),
        capacity: Set(50),
        public_id: Set(Uuid::new_v4()),
        ..Default::default()
    }
    .insert(txn)
    .await?;

    rooms.push(room1);
    rooms.push(room2);

    Ok(rooms)
}

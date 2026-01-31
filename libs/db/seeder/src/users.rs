use entity::user;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DbErr};
use uuid::Uuid;

/// Seeds initial users into the database.
pub async fn seed_users(txn: &sea_orm::DatabaseTransaction) -> Result<Vec<user::Model>, DbErr> {
    let mut users = Vec::new();

    let user1 = user::ActiveModel {
        name: Set("seeder_alice".to_owned()),
        status: Set(Some("online".to_owned())),
        room: Set(None),
        public_id: Set(Uuid::new_v4()),
        ..Default::default()
    }
    .insert(txn)
    .await?;

    let user2 = user::ActiveModel {
        name: Set("seeder_bob".to_owned()),
        status: Set(Some("offline".to_owned())),
        room: Set(None),
        public_id: Set(Uuid::new_v4()),
        ..Default::default()
    }
    .insert(txn)
    .await?;

    users.push(user1);
    users.push(user2);

    Ok(users)
}

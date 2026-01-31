use sea_orm::{DatabaseConnection, DbErr, TransactionError, TransactionTrait};
mod messages;
mod rooms;
mod users;

/// Seeds all data into the database within a transaction.
pub async fn seed_all(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.transaction(|txn| {
        Box::pin(async move {
            let rooms = rooms::seed_rooms(txn).await?;
            let users = users::seed_users(txn).await?;
            messages::seed_messages(txn, &users, &rooms).await?;
            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(db_err) => db_err,
        TransactionError::Transaction(db_err) => db_err,
    })
}

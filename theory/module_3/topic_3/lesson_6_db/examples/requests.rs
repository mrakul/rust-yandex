use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};

// 1. cargo install sqlx-cli --no-default-features --features rustls,postgres
// 2. export DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/practice_db
// 3. cargo sqlx prepare --all (?)
// Не получилось добиться, чтобы сработало на ./examples. Скорее всего, надо переносить в ./src,
// чтобы сработало

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    id: i32,
    balance: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
} 

// ✅ Compile-time проверка с query_as!
async fn get_user(pool: &PgPool, id: uuid::Uuid) -> Result<Option<User>, sqlx::Error> {
    let account = sqlx::query_as!(
        User,
        "SELECT id, email, created_at FROM users WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(account)
}

fn main() {
    todo!()
}

// // ✅ Compile-time проверка с query_as!
// async fn get_account(pool: &PgPool, account_id: i32) -> Result<Option<Account>, sqlx::Error> {
//     let account = sqlx::query_as!(
//         Account,
//         "SELECT id, balance, created_at FROM accounts WHERE id = $1",
//         account_id
//     )
//     .fetch_optional(pool)
//     .await?;
    
//     Ok(account)
// }

// // ✅ Compile-time проверка с query! (анонимная структура)
// async fn get_balance(pool: &PgPool, account_id: i32) -> Result<Option<i64>, sqlx::Error> {
//     let row = sqlx::query!(
//         "SELECT balance FROM accounts WHERE id = $1",
//         account_id
//     )
//     .fetch_optional(pool)
//     .await?;
    
//     Ok(row.map(|r| r.balance))
// }

// // ✅ INSERT с RETURNING
// async fn create_account(pool: &PgPool, initial_balance: i64) -> Result<Account, sqlx::Error> {
//     let account = sqlx::query_as!(
//         Account,
//         "INSERT INTO accounts (balance) VALUES ($1) RETURNING id, balance, created_at",
//         initial_balance
//     )
//     .fetch_one(pool)
//     .await?;
    
//     Ok(account)
// }

// // ✅ UPDATE
// async fn update_balance(pool: &PgPool, account_id: i32, new_balance: i64) -> Result<(), sqlx::Error> {
//     sqlx::query!(
//         "UPDATE accounts SET balance = $1 WHERE id = $2",
//         new_balance,
//         account_id
//     )
//     .execute(pool)
//     .await?;
    
//     Ok(())
// }

// // ✅ DELETE
// async fn delete_account(pool: &PgPool, account_id: i32) -> Result<bool, sqlx::Error> {
//     let result = sqlx::query!(
//         "DELETE FROM accounts WHERE id = $1",
//         account_id
//     )
//     .execute(pool)
//     .await?;
    
//     Ok(result.rows_affected() > 0)
// }

// use sqlx::Acquire;

// // ✅ Транзакция для перевода денег
// async fn transfer_money(
//     pool: &PgPool,
//     from_id: i32,
//     to_id: i32,
//     amount: i64,
// ) -> Result<(), sqlx::Error> {
//     let mut tx = pool.begin().await?;
    
//     // Списываем с первого счёта
//     sqlx::query!(
//         "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
//         amount,
//         from_id
//     )
//     .execute(&mut *tx)
//     .await?;
    
//     // Зачисляем на второй счёт
//     sqlx::query!(
//         "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
//         amount,
//         to_id
//     )
//     .execute(&mut *tx)
//     .await?;
    
//     // Если всё ок — коммитим
//     tx.commit().await?;
//     Ok(())
    
//     // При ошибке автоматически выполнится rollback
// }
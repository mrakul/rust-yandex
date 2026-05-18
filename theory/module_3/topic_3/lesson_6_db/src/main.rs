use sqlx::{postgres::PgPoolOptions, PgPool, PgConnection, Connection};
use std::env;

// ✅ Создание пула соединений с настройками
async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost/bank_db".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await?;
    
    Ok(pool)
}

// ✅ Одно соединение (для простых случаев)
async fn create_connection() -> Result<PgConnection, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost/bank_db".to_string());
    
    let conn = PgConnection::connect(&database_url).await?;
    Ok(conn)
}

fn main() {
    println!("Hello, world!");
}

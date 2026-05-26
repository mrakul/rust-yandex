use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

// Из примера bank-api + Postgres

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        // .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(database_url)
        .await?;
    
    info!("connected to PostgreSQL");
    
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("running database migrations");
    sqlx::migrate!().run(pool).await?;
    info!("migrations completed");

    Ok(())
}
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use tracing::{error, info};

use crate::domain::{user::User, DomainError};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> Result<User, DomainError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
}

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Основано на примере create из bank-api + Postgress
#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, mut user: User) -> Result<User, DomainError> {
        // TODO: поменять на query! для проверки на этапе компиляции
        let row = sqlx::query(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at
            "#,
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        // INSERT должен вернуть одну запись
        .fetch_one(&self.pool)
        .await

        .map_err(|e| {
            error!("НЕвозможно создать пользователя: {}", e);
            if e.as_database_error()
                .and_then(|db| db.constraint())
                .map(|c| c.contains("users_username") || c.contains("users_email")) == Some(true) 
            {
                DomainError::UserAlreadyExists(user.username.clone())
            } else {
                DomainError::Internal(format!("database error: {}", e))
            }
        })?;

        user.id = row.get("id");
        info!(user_id = user.id, username = %user.username, "user created");
        
        Ok(user)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        // Может не быть
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Пользователь не найден {}: {}", id, e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;

        Ok(row.map(map_row))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        // Может не быть
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Ошибка поиска по имени {}: {}", username, e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;

        Ok(row.map(map_row))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        // Может не быть
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Ошибка поиска по e-mail {}: {}", email, e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;

        Ok(row.map(map_row))
    }
}

// Отдельная функция для маппинга в User
fn map_row(row: sqlx::postgres::PgRow) -> User {
    User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        created_at: row.get("created_at"),
    }
}
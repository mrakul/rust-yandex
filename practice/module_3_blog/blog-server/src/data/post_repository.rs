use async_trait::async_trait;
use sqlx::{PgPool, Row};
use tracing::{error, info};

use crate::domain::{post::Post, DomainError};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: Post) -> Result<Post, DomainError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, DomainError>;
    async fn update(&self, post: Post) -> Result<Post, DomainError>;
    async fn delete(&self, id: i64) -> Result<(), DomainError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Post>, DomainError>;
    async fn count(&self) -> Result<i64, DomainError>;
}

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, mut post: Post) -> Result<Post, DomainError> {
        let row = sqlx::query(
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.author_id)
        // Должен вернуть одну запись
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Ошибка создания поста: {}", e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;

        post.id = row.get("id");
        info!(post_id = post.id, author_id = post.author_id, "пост создан");

        Ok(post)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Пост не найден по ID {}: {}", id, e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;

        Ok(row.map(map_row))
    }

    async fn update(&self, post: Post) -> Result<Post, DomainError> {
        let row = sqlx::query(
            r#"
            UPDATE posts
            SET title = $1, content = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Не удалось обновить пост по ID {}: {}", post.id, e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;

        match row {
            Some(row) => {
                info!(post_id = post.id, "пост обновлён");
                Ok(map_row(row))
            }
            None => Err(DomainError::PostNotFound(post.id)),
        }
    }

    async fn delete(&self, id: i64) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"DELETE FROM posts WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Не удалось удалить пост по ID {}: {}", id, e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;

        if result.rows_affected() == 0 {
            Err(DomainError::PostNotFound(id))
        } else {
            info!(post_id = id, "пост удалён");
            Ok(())
        }
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Post>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        // Тут все записи с учётом bind'ов
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Ошибка получения списка: {}", e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;

        Ok(rows.into_iter().map(map_row).collect())
    }

    async fn count(&self) -> Result<i64, DomainError> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM posts"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Ошибка подсчёта постов: {}", e);
            DomainError::Internal(format!("DB error: {}", e))
        })?;
        Ok(count)
    }
}

// Аналогично User - маппинг в Post записи
fn map_row(row: sqlx::postgres::PgRow) -> Post {
    Post {
        id: row.get("id"),
        title: row.get("title"),
        content: row.get("content"),
        author_id: row.get("author_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    // Пароль убираем из сериализации
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        Self {
            // Устанавливается БД
            id: 0,
            username,
            email,
            password_hash,
            created_at: Utc::now(),
        }
    }
}
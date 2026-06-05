// Вынес отдельно DTO
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String
}

// Логин отдельно
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserDto
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserDto {
    id: i64,
    // Сохранякем только username
    pub username: String,
    email: String
}

// Посты
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostPublic {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ListPostsResponse {
    pub posts: Vec<PostPublic>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64
}
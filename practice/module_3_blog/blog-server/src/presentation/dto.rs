use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::{user::User, post::Post};

// === Auth DTOs ===
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

// === Post DTOs ===
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>
}

#[derive(Debug, Serialize)]
pub struct PostPublic {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl From<Post> for PostPublic {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListPostsResponse {
    pub posts: Vec<PostPublic>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64
}

// === Health DTO ===
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: DateTime<Utc>
}
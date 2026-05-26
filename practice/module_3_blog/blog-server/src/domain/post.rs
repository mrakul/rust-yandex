use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    // Оба опциональны
    pub title: Option<String>,
    pub content: Option<String>,
}

impl Post {
    pub fn new(author_id: i64, title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            title,
            content,
            author_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, title: Option<String>, content: Option<String>) {
        // Обрабатываем как Option только по Some
        if let Some(title_option) = title {
            self.title = title_option;
        }

        if let Some(content_option) = content {
            self.content = content_option;
        }

        self.updated_at = Utc::now();
    }
}
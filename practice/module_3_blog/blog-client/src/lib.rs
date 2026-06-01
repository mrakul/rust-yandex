//! Blog client library supporting both HTTP and gRPC transports.
//! Provides unified API for interacting with the blog server.

mod error;
mod http_client;
// mod grpc_client;

pub use error::BlogClientError;
pub use http_client::HttpClient;
// pub use grpc_client::GrpcClient;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    
    pub created_at: DateTime<Utc>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPostsResponse {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64
}

// Enum для задания протокола
#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),           // HTTP-сервер здесь: http://localhost:3000
    Grpc(String)            // gRPC-сервер здесь: http://localhost:50051
}

// Клиент: тип протокола, HTTP/gRPC, токен
#[derive(Debug, Clone)]
pub struct BlogClient {
    transport: Transport,
    http_client: Option<HttpClient>,
    // grpc_client: Option<GrpcClient>,
    token: Option<String>
}

impl BlogClient {
    // Создание нового клиента в зависимости от транспорта
    pub fn new(transport: Transport) -> Result<Self, BlogClientError> {
        match &transport {
            Transport::Http(base_url) => {
                let http_client = HttpClient::new(base_url.clone())?;
                Ok(Self {
                    transport,
                    http_client: Some(http_client),
                    // grpc_client: None,
                    token: None,
                })
            }

            Transport::Grpc(_addr) => {
                // TODO: gRPC пока заглушка
                Err(BlogClientError::UnsupportedTransport("gRPC пока не реализован".into()))
            }
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token.clone());

        // Мутабельная ссылка
        if let Some(http_client) = &mut self.http_client {
            http_client.set_token(token);
        }
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    // Регистрация
    pub async fn register(&self,
                          username: String,
                          email: String,
                          password: String) -> Result<AuthResponse, BlogClientError>
    {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.register(username, email, password).await
                } else {
                    // Например, пока не сделана gRPC часть
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }

            // TODO: gRPC пока заглушка
            Transport::Grpc(_) => Err(BlogClientError::UnsupportedTransport("gRPC пока не реализован".into())),
        }
    }

    pub async fn login(&self,
                       username: String,
                       password: String) -> Result<AuthResponse, BlogClientError>
    {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.login(username, password).await
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }

            // TODO: gRPC пока заглушка
            Transport::Grpc(_) => Err(BlogClientError::UnsupportedTransport("gRPC пока не реализован".into())),
        }
    }

    /*** Методы постов ***/

    pub async fn create_post(&self,
                             title: String,
                             content: String) -> Result<Post, BlogClientError>
    {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.create_post(title, content).await
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }
            Transport::Grpc(_) => Err(BlogClientError::UnsupportedTransport("gRPC пока не реализован".into())),
        }
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.get_post(id).await
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }
            Transport::Grpc(_) => Err(BlogClientError::UnsupportedTransport("gRPC пока не реализован".into())),
        }
    }

    pub async fn update_post(&self,
                             id: i64,
                             title: Option<String>,
                             content: Option<String>) -> Result<Post, BlogClientError>
    {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.update_post(id, title, content).await
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }
            Transport::Grpc(_) => Err(BlogClientError::UnsupportedTransport("gRPC пока не реализован".into())),
        }
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.delete_post(id).await
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }
            Transport::Grpc(_) => Err(BlogClientError::UnsupportedTransport("gRPC пока не реализован".into())),
        }
    }

    pub async fn list_posts(&self,
                            limit: Option<i64>,
                            offset: Option<i64>) -> Result<ListPostsResponse, BlogClientError>
    {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.list_posts(limit, offset).await
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }
            Transport::Grpc(_) => Err(BlogClientError::UnsupportedTransport("gRPC пока не реализован".into())),
        }
    }
}
mod error;
mod http_client;
mod grpc_client;

pub use error::BlogClientError;
pub use http_client::HttpClient;
pub use grpc_client::GrpcClient;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Включаем сгенерированный код из build.rs
pub mod generated {
    tonic::include_proto!("blog");
}


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
#[derive(Clone)]
pub struct BlogClient {
    transport: Transport,
    http_client: Option<HttpClient>,
    grpc_client: Option<GrpcClient>,
    token: Option<String>
}

impl BlogClient {
    // Создание нового клиента в зависимости от транспорта
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let (http_client, grpc_client) = match &transport {
            Transport::Http(server_addr) => {
                // Тут важно, что ?, чтобы вернуть "распакованный" Result
                (Some(HttpClient::new(server_addr.clone())?), None)
            }
            Transport::Grpc(server_addr) => {
                // Делаем connect при создании клиента
                let grpc_client = GrpcClient::new(server_addr.clone()).await?;
                (None, Some(grpc_client))
            }
        };

        // В Option или один, или другой тип клиента
        Ok(Self {
            transport,
            http_client,
            grpc_client,
            token: None,
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token.clone());

        // Мутабельная ссылка
        if let Some(http_client) = &mut self.http_client {
            http_client.set_token(token.clone());
        }

        // Аналогично для gRPC
        if let Some(grpc) = &mut self.grpc_client {
            grpc.set_token(token);
        }
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    // Регистрация
    pub async fn register(&mut self,
                          username: String,
                          email: String,
                          password: String) -> Result<AuthResponse, BlogClientError>
    {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                   let auth_response = client.register(username, email, password).await?;
                   self.set_token(auth_response.token.clone());
                   Ok(auth_response)
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }
            // TODO: gRPC пока заглушка
            Transport::Grpc(_) => {
                if let Some(grpc_client) = &mut self.grpc_client {
                    let auth_response = grpc_client.register(username, email, password).await?;
                // Сохраняем токен - сделал внутри
                    // self.set_token(auth_response.token.clone());
                    Ok(auth_response)
                } else {
                    Err(BlogClientError::InvalidState("gRPC-клиент не инициализирован".into()))
                }
            }
        }
    }

    pub async fn login(&mut self,
                       username: String,
                       password: String) -> Result<AuthResponse, BlogClientError>
    {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &mut self.http_client { // Use &mut for http client
                    let auth_response = client.login(username, password).await?;
                    self.set_token(auth_response.token.clone());
                    Ok(auth_response)
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }

            Transport::Grpc(_) => {
                if let Some(client) = &mut self.grpc_client {
                    let auth_response = client.login(username, password).await?;
                    self.set_token(auth_response.token.clone());
                    Ok(auth_response)
                } else {
                    Err(BlogClientError::InvalidState("gRPC-клиент не инициализирован".into()))
                }
            }
        }
    }

    /*** Методы постов ***/

    pub async fn create_post(&mut self,
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
            Transport::Grpc(_) => {
                if let Some(client) = &mut self.grpc_client {
                    client.create_post(title, content).await
                } else {
                    Err(BlogClientError::InvalidState("gRPC-клиент не инициализирован".into()))
                }
            }
        }
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.get_post(id).await
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &mut self.grpc_client {
                    client.get_post(id).await
                } else {
                    Err(BlogClientError::InvalidState("gRPC-клиент не инициализирован".into()))
                }
            }
        }
    }

    pub async fn update_post(&mut self,
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
            Transport::Grpc(_) => {
                if let Some(client) = &mut self.grpc_client {
                    client.update_post(id, title, content).await
                } else {
                    Err(BlogClientError::InvalidState("gRPC-клиент не инициализирован".into()))
                }
            }
        }
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                if let Some(client) = &self.http_client {
                    client.delete_post(id).await
                } else {
                    Err(BlogClientError::InvalidState("HTTP-клиент не инициализирован".into()))
                }
            }
            Transport::Grpc(_) => {
                if let Some(client) = &mut self.grpc_client {
                    client.delete_post(id).await 
                } else {
                    Err(BlogClientError::InvalidState("gRPC-клиент не инициализирован".into()))
                }
            }
        }
    }

    pub async fn list_posts(&mut self,
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
            Transport::Grpc(_) => {
                if let Some(client) = &mut self.grpc_client {
                    client.list_posts(limit.unwrap_or(10), offset.unwrap_or(0)).await
                } else {
                    Err(BlogClientError::InvalidState("gRPC-клиент не инициализирован".into()))
                }
            }
        }
    }
}
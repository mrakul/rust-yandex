use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
// use std::collections::HashMap;

use crate::{BlogClientError, AuthResponse, Post, User, ListPostsResponse};

#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    client: Client,
    token: Option<String>
}

// DTO
#[derive(Serialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String
}

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String
}

#[derive(Serialize)]
struct CreatePostRequest {
    title: String,
    content: String
}

#[derive(Serialize)]
struct UpdatePostRequest {
    title: Option<String>,
    content: Option<String>
}

impl HttpClient {
    pub fn new(base_url: String) -> Result<Self, BlogClientError> {
        // Из примера урока 3 "Bank-API + Postgres"
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))    
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| BlogClientError::InitializationError(e.to_string()))?;

        // Токен отсутствует до регистрации/логина
        Ok(Self {base_url, client, token: None})
    }

    // Для CLI будем читать из файла, тут, наверное, не сильно принципиально
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }


    // Регистрация
    pub async fn register(&self,
                          username: String,
                          email: String,
                          password: String) -> Result<AuthResponse, BlogClientError>
    {
        let request = self.build_request(reqwest::Method::POST, "/auth/register");
        
        let response = request
            .json(&RegisterRequest {username,
                                    email,
                                    password})
            .send()
            .await
            .map_err(|e| BlogClientError::NetworkError(e.to_string()))?;

        match response.status() {
            StatusCode::CREATED => {
                let auth_response: AuthResponse = response
                    .json()
                    .await
                    .map_err(|e| BlogClientError::SerializationError(e.to_string()))?;

                Ok(auth_response)
            }

            StatusCode::CONFLICT => Err(BlogClientError::UserAlreadyExists),
            StatusCode::BAD_REQUEST => Err(BlogClientError::ValidationError(
                response.text().await.unwrap_or_else(|_| "Bad request".to_string()))),
            
            status => Err(BlogClientError::UnexpectedStatus(status.as_u16())),
        }
    }

    pub async fn login(&self,
                       username: String,
                       password: String) -> Result<AuthResponse, BlogClientError>
    {
        let request = self.build_request(reqwest::Method::POST, "/auth/login");
        
        let response = request
            .json(&LoginRequest {
                username,
                password})
            .send()
            .await
            .map_err(|e| BlogClientError::NetworkError(e.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let auth_response: AuthResponse = response
                    .json()
                    .await
                    .map_err(|e| BlogClientError::SerializationError(e.to_string()))?;
                Ok(auth_response)
            }
            StatusCode::UNAUTHORIZED => Err(BlogClientError::InvalidCredentials),
            StatusCode::BAD_REQUEST => Err(BlogClientError::ValidationError(
                response.text().await.unwrap_or_else(|_| "Bad request".to_string()),
            )),
            status => Err(BlogClientError::UnexpectedStatus(status.as_u16())),
        }
    }

    /*** Методы постов ***/

    pub async fn create_post(&self, 
                             title: String,
                             content: String) -> Result<Post, BlogClientError>
    {
        let request = self.build_request(reqwest::Method::POST, "/posts");
        
        let response = request
            .json(&CreatePostRequest { title, content })
            .send()
            .await
            .map_err(|e| BlogClientError::NetworkError(e.to_string()))?;

        match response.status() {
            StatusCode::CREATED => {
                let post: Post = response
                    .json()
                    .await
                    .map_err(|e| BlogClientError::SerializationError(e.to_string()))?;

                Ok(post)
            }
            StatusCode::UNAUTHORIZED => Err(BlogClientError::Unauthorized),
            StatusCode::FORBIDDEN => Err(BlogClientError::Forbidden),
            StatusCode::BAD_REQUEST => Err(BlogClientError::ValidationError(
                response.text().await.unwrap_or_else(|_| "Bad request".to_string()),
            )),

            status => Err(BlogClientError::UnexpectedStatus(status.as_u16())),
        }
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let request = self.build_request(reqwest::Method::GET, &format!("/posts/{}", id));
        
        let response = request.send().await.map_err(|e| BlogClientError::NetworkError(e.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let post: Post = response
                    .json()
                    .await
                    .map_err(|e| BlogClientError::SerializationError(e.to_string()))?;
                Ok(post)
            }
            StatusCode::NOT_FOUND => Err(BlogClientError::PostNotFound(id)),
            status => Err(BlogClientError::UnexpectedStatus(status.as_u16())),
        }
    }

    pub async fn update_post(&self,
                            id: i64,
                            title: Option<String>,
                            content: Option<String>) -> Result<Post, BlogClientError>
    {
        // Здесь PUT-запрос
        let request = self.build_request(reqwest::Method::PUT, &format!("/posts/{}", id));
        
        let response = request
            .json(&UpdatePostRequest { title, content })
            .send()
            .await
            .map_err(|e| BlogClientError::NetworkError(e.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let post: Post = response
                    .json()
                    .await
                    .map_err(|e| BlogClientError::SerializationError(e.to_string()))?;
                Ok(post)
            }
            StatusCode::UNAUTHORIZED => Err(BlogClientError::Unauthorized),
            StatusCode::FORBIDDEN => Err(BlogClientError::Forbidden),
            StatusCode::NOT_FOUND => Err(BlogClientError::PostNotFound(id)),
            StatusCode::BAD_REQUEST => Err(BlogClientError::ValidationError(
                response.text().await.unwrap_or_else(|_| "Bad request".to_string()),
            )),

            status => Err(BlogClientError::UnexpectedStatus(status.as_u16())),
        }
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        // DELETE
        let request = self.build_request(reqwest::Method::DELETE, &format!("/posts/{}", id));
        
        let response = request.send().await.map_err(|e| BlogClientError::NetworkError(e.to_string()))?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::UNAUTHORIZED => Err(BlogClientError::Unauthorized),
            StatusCode::FORBIDDEN => Err(BlogClientError::Forbidden),
            StatusCode::NOT_FOUND => Err(BlogClientError::PostNotFound(id)),
            status => Err(BlogClientError::UnexpectedStatus(status.as_u16())),
        }
    }

    // Список с пагинацией
    pub async fn list_posts(&self,
                            limit_opt: Option<i64>,
                            offset_opt: Option<i64>) -> Result<ListPostsResponse, BlogClientError>
    {
        // Список параметров через вектор
        let mut query_parts = Vec::new();
        
        if let Some(limit) = limit_opt {
            query_parts.push(format!("limit={}", limit));
        }

        if let Some(offset) = offset_opt {
            query_parts.push(format!("offset={}", offset));
        }
        
        // По-простому: '?' для первого параметра и '&' для последующих
        let query_string = if !query_parts.is_empty() {
            format!("?{}", query_parts.join("&"))
        } else {
            String::new()
        };

        // Подставляем запрос
        let url_with_query = format!("{}{}", self.build_full_url("/posts"), query_string);

        let mut request = self.client.request(reqwest::Method::GET, url_with_query);

        // Ссылка: можно ref token или self.token.as_ref()
        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request = request.header("Content-Type", "application/json");
        
        let response = request.send().await.map_err(|e| BlogClientError::NetworkError(e.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let list_response: ListPostsResponse = response
                    .json()
                    .await
                    .map_err(|e| BlogClientError::SerializationError(e.to_string()))?;
                Ok(list_response)
            }
            
            StatusCode::BAD_REQUEST => {
                let error_text = response.text().await
                    .unwrap_or_else(|_| "Bad request".to_string());
                Err(BlogClientError::ValidationError(error_text))
            },

            status => Err(BlogClientError::UnexpectedStatus(status.as_u16())),
        }
    }


    // Вспомогательные функции
    fn build_full_url(&self, path: &str) -> String {
        format!("{}/api{}", self.base_url, path)
    }

    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder
    {
        let url = self.build_full_url(path);
        let mut request = self.client.request(method, url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.header("Content-Type", "application/json")
    }

    // Более широкий вариант с использованием urlencoding для преобразования в % коды и обратно
    // fn build_query_string(params: &HashMap<String, String>) -> String {
    //     if params.is_empty() {
    //         return String::new();
    //     }

    //     let query_pairs: Vec<String> = params
    //         .iter()
    //         .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
    //         .collect();
        
    //     format!("?{}", query_pairs.join("&"))
    // }
}


use tonic::metadata::MetadataValue;

use crate::generated::blog_service_client::BlogServiceClient;
use crate::generated::{ RegisterRequest, LoginRequest, CreatePostRequest, GetPostRequest,
                        UpdatePostRequest, DeletePostRequest, ListPostsRequest
};

use crate::{AuthResponse, Post, ListPostsResponse, BlogClientError};

#[derive(Clone)]
pub struct GrpcClient {
    // Как в уроке 3, тема 7
    client: BlogServiceClient<tonic::transport::Channel>,
    token: Option<String>
}

impl GrpcClient {
    pub async fn new(address: String) -> Result<Self, BlogClientError> {
        let client = BlogServiceClient::connect(address)
            .await
            .map_err(|error| BlogClientError::GrpcError(error.to_string()))?;
        
        // Возвращаем клиента без токена
        Ok(Self {
            client,
            token: None
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    // Добавляем authorization к запросам, generic для разных запросов
    fn add_auth_to_request<ReqType>(&self, mut request: tonic::Request<ReqType>) -> Result<tonic::Request<ReqType>, BlogClientError> {
        if let Some(ref token) = self.token {
            // КОнвертируем String -> tonic::metadata::value::MetadataValue
            let auth_bearer_ascii = MetadataValue::try_from(format!("Bearer {}", token))
                .map_err(|_| BlogClientError::InvalidState("Неверный формат токена".into()))?;
            // И вставляем в запрос в формате: ("authorization", "Bearer <token>")
            request.metadata_mut().insert("authorization", auth_bearer_ascii);
        }

        Ok(request)
    }

    // Регистрация
    pub async fn register(&mut self, username: String, email: String, password: String) -> Result<AuthResponse, BlogClientError> {
        let register_request = tonic::Request::new(RegisterRequest {
            username,
            email,
            password
        });

        let grpc_auth_response = self.client.register(register_request)
            .await
            .map_err(|e| BlogClientError::GrpcError(e.message().to_string()))?
            // Это "расчехляет" Response: AuthResponse из Response<AuthResponse> и в остальных запросах так же
            .into_inner();

        let user = grpc_auth_response.user.ok_or_else(|| BlogClientError::InvalidState("Отсутствует User в ответе".into()))?;
        
        // => AuthResponse
        let auth_resp = AuthResponse {
            token: grpc_auth_response.token,
            user: crate::User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.parse().unwrap_or_default()
            },
        };

        // Сохраняем токен
        self.set_token(auth_resp.token.clone());
        
        Ok(auth_resp)
    }

    // Логин
    pub async fn login(&mut self, username: String, password: String) -> Result<AuthResponse, BlogClientError> {
        let login_request = tonic::Request::new(LoginRequest {
            username,
            password,
        });

        let grpc_auth_response = self.client.login(login_request)
            .await
            .map_err(|e| BlogClientError::GrpcError(e.message().to_string()))?
            .into_inner();

        let gprc_user = grpc_auth_response.user.ok_or_else(|| BlogClientError::InvalidState("Отсутствует User в ответе".into()))?;

        let auth_resp = AuthResponse {
            token: grpc_auth_response.token,
            user: crate::User {
                id: gprc_user.id,
                username: gprc_user.username,
                email: gprc_user.email,
                created_at: gprc_user.created_at.parse().unwrap_or_default()
            },
        };

        // Сохраняем токен
        self.set_token(auth_resp.token.clone());
        
        Ok(auth_resp)
    }

    // Создание поста
    pub async fn create_post(&mut self, title: String, content: String) -> Result<Post, BlogClientError> {
        let create_post_request = tonic::Request::new(CreatePostRequest {
            title,
            content
        });
        
        // С хедером авторизации
        let create_post_request = self.add_auth_to_request(create_post_request)?;

        let post_response = self.client.create_post(create_post_request)
            .await
            .map_err(|e| BlogClientError::GrpcError(e.message().to_string()))?
            .into_inner();

        let grpc_post = post_response.post.ok_or_else(|| BlogClientError::InvalidState("Отсутствует пост в ответе".into()))?;

        Ok( Post{
            id: grpc_post.id,
            title: grpc_post.title,
            content: grpc_post.content,
            author_id: grpc_post.author_id,
            created_at: grpc_post.created_at.parse().unwrap_or_default(),
            updated_at: grpc_post.updated_at.parse().unwrap_or_default()})
    }

    // Получить пост по ID
    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let get_post_request = tonic::Request::new(GetPostRequest { id });

        let post_response = self.client.get_post(get_post_request)
            .await
            .map_err(|e| BlogClientError::GrpcError(e.message().to_string()))?
            .into_inner();

        let grpc_post = post_response.post.ok_or_else(|| BlogClientError::PostNotFound(id))?;

        Ok(Post {
            id: grpc_post.id,
            title: grpc_post.title,
            content: grpc_post.content,
            author_id: grpc_post.author_id,
            created_at: grpc_post.created_at.parse().unwrap_or_default(),
            updated_at: grpc_post.updated_at.parse().unwrap_or_default(),
        })
    }

    // Update поста
    pub async fn update_post(&mut self, id: i64, title: Option<String>, content: Option<String>) -> Result<Post, BlogClientError> {
        let title = title.unwrap_or_default();
        let content = content.unwrap_or_default();

        let update_post_request = tonic::Request::new(UpdatePostRequest{id, title, content});
        
        // Пришлёпываем авторизацию
        let update_post_request = self.add_auth_to_request(update_post_request)?;

        let post_response_grpc = self.client.update_post(update_post_request)
            .await
            .map_err(|e| BlogClientError::GrpcError(e.message().to_string()))?
            .into_inner();

        let post_grpc = post_response_grpc.post.ok_or_else(|| BlogClientError::InvalidState("Отсутствует пост в ответе".into()))?;

        Ok(Post {
            id: post_grpc.id,
            title: post_grpc.title,
            content: post_grpc.content,
            author_id: post_grpc.author_id,
            created_at: post_grpc.created_at.parse().unwrap_or_default(),
            updated_at: post_grpc.updated_at.parse().unwrap_or_default(),
        })
    }

    // Удаление
    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let delete_request = tonic::Request::new(DeletePostRequest { id });
        let delete_request = self.add_auth_to_request(delete_request)?;

        self.client.delete_post(delete_request)
            .await
            .map_err(|e| BlogClientError::GrpcError(e.message().to_string()))?;

        Ok(())
    }

    // Список постов
    pub async fn list_posts(&mut self, limit: i64, offset: i64) -> Result<ListPostsResponse, BlogClientError> {
        let list_request = tonic::Request::new(ListPostsRequest {
            limit: limit as i32,
            offset: offset as i32
        });

        let list_response_grpc = self.client.list_posts(list_request)
            .await
            .map_err(|e| BlogClientError::GrpcError(e.message().to_string()))?
            .into_inner();

        // Собираем в вектор
        let posts = list_response_grpc.posts.into_iter().map(|post| Post {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.parse().unwrap_or_default(),
            updated_at: post.updated_at.parse().unwrap_or_default(),
        }).collect();

        Ok(ListPostsResponse {
            posts,
            total: list_response_grpc.total,
            limit: list_response_grpc.limit as i64,
            offset: list_response_grpc.offset as i64,
        })
    }
}
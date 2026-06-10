
use tonic::{Request, Response, Status};

use crate::application::{AuthService, BlogService};
use crate::data::{PostgresUserRepository, PostgresPostRepository};
use crate::infrastructure::jwt::JwtService;

use crate::presentation::generated::blog_service_server::BlogService as BlogServiceServer;
// Импорт сгенерированных типов из ProtoBuf
use crate::presentation::generated::{ RegisterRequest, LoginRequest, AuthResponse, User, 
                                      CreatePostRequest, PostResponse, Post, GetPostRequest, 
                                       UpdatePostRequest, DeletePostRequest, DeletePostResponse,
                                      ListPostsRequest, ListPostsResponse
                                    };
// Логирование
use tracing::info;


// Сервисы для gRPC
#[derive(Clone)]
pub struct BlogGrpcService {
    auth_service: AuthService<PostgresUserRepository>,
    blog_service: BlogService<PostgresPostRepository>,
    jwt_service: JwtService,
}

impl BlogGrpcService {
    pub fn new(auth_service: AuthService<PostgresUserRepository>,
               blog_service: BlogService<PostgresPostRepository>,
               jwt_service: JwtService) -> Self 
    {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }

    // Вспомогательная функция для извлечения токена из метаданных
    // (!) Сделал generic для любого типа запроса
    fn extract_token<ReqType>(&self, request: &Request<ReqType>) -> Result<String, Status> {
        let metadata = request.metadata();
        if let Some(auth_header) = metadata.get("authorization") {
            
            // Токен c Bearer в &str
            let token_str = auth_header
                .to_str().
                map_err(|_| {
                    Status::unauthenticated("Неверный формат токена")
                })?;
            
            // Отрезаем бирер
            if let Some(token) = token_str.strip_prefix("Bearer ") {
                Ok(token.to_string())
            } else {
                Err(Status::unauthenticated("Bearer отсутствует"))
            }
        } else {
            Err(Status::unauthenticated("Нет хедера авторизации"))
        }
    }

    // Вспомогательная функция для проверки токена и получения user_id
    // (!) Аналогично, сделал generic для любого типа запроса
    async fn verify_auth<ReqType>(&self, request: &Request<ReqType>) -> Result<i64, Status> {
        let token = self.extract_token(request)?;
        let claims = self.jwt_service.verify_token(&token)
            .map_err(|_| Status::unauthenticated("Неверный токен"))?;
        
        claims.sub.parse::<i64>()
             .map_err(|_| Status::unauthenticated("Некорректный user ID в токене"))
    }
}

#[tonic::async_trait]
impl BlogServiceServer for BlogGrpcService {

    /*** Public API ***/

    // Регистрация: RegisterRequest -> AuthResponse
    async fn register(&self,  request: Request<RegisterRequest>) -> Result<Response<AuthResponse>, Status>
    {
        let register_request = request.into_inner();
        info!("gRPC регистрация, пользователь: {}", register_request.username);

        let (user, token) = self.auth_service.register(register_request.username, register_request.email, register_request.password)
            .await
            .map_err(|error| {
                match error {
                    // TODO: действительно, тут бы сделать mapping BlogError => Status и везде ниже
                    // Conflict 409 и Validation
                    crate::domain::BlogError::Conflict(_) => Status::already_exists("Пользователь уже существует"),
                    crate::domain::BlogError::Validation(msg) => Status::invalid_argument(msg),
                    _ => Status::internal("Внутренняя ошибка регистрации"),
                }
            })?;

        // Получаем токен при регистрации тоже, как и при HTTP, и переводв ProtoBuf User из Domain
        let response = AuthResponse {
            token,
            user: Some(User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
            }),
        };

        Ok(Response::new(response))
    }

    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<AuthResponse>, Status>
    {
        let login_request = request.into_inner();
        info!("gRPC логин, пользователь: {}", login_request.username);

        let (user, token) = self.auth_service.login(login_request.username, login_request.password)
            .await
            .map_err(|error| {
                match error {
                    // Unauthorized 401
                    crate::domain::BlogError::Unauthorized => Status::unauthenticated("Неверные креды"),
                    _ => Status::internal("Login failed"),
                }
            })?;
        // => ProtoBuf User
        let response = AuthResponse {
            token,
            user: Some(User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
            }),
        };

        Ok(Response::new(response))
    }

    // Список постов
    async fn list_posts(&self, request: Request<ListPostsRequest>) -> Result<Response<ListPostsResponse>, Status> 
    {
        let list_posts_request = request.into_inner();
        // limit, offset
        let limit = list_posts_request.limit as i64;
        let offset = list_posts_request.offset as i64;

        // Вектор с количеством, как для HTTP (разумеется)
        let (posts, total) = self.blog_service.list_posts(limit, offset)
            .await
            .map_err(|_| Status::internal("Ошибка получения списка постов"))?;

        // => ProtoBuf Post
        let grpc_posts_vec = posts.into_iter()
            .map(|post| Post {
                id: post.id,
                 title: post.title,
                content: post.content,
                author_id: post.author_id,
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
        }).collect();

        // Весь респонс
        let response = ListPostsResponse {
            posts: grpc_posts_vec,
            total,
            limit: limit as i32,
            offset: offset as i32,
        };

        Ok(Response::new(response))
    }


    async fn get_post(&self, request: Request<GetPostRequest>) -> Result<Response<PostResponse>, Status> 
    {
        let get_post_request = request.into_inner();
        
        let post = self.blog_service.get_post(get_post_request.id)
            .await
            .map_err(|error| {
                match error {
                    // 404
                    crate::domain::BlogError::NotFound(_) => Status::not_found("Пост не найден"),
                    _ => Status::internal("Внутренняя ошибка получения поста"),
                }
            })?;

        // => ProtoBuf Post
        let response = PostResponse {
            post: Some(Post {
                id: post.id,
                title: post.title,
                content: post.content,
                author_id: post.author_id,
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            }),
        };

        Ok(Response::new(response))
    }

    /*** Protected API, посты, с проверкой авторизации ***/

    // Создание поста
    async fn create_post(&self, request: Request<CreatePostRequest>) -> Result<Response<PostResponse>, Status>
    {
        // Авторизацию
        let user_id = self.verify_auth(&request).await?;
        
        let create_post_req = request.into_inner();
        info!("gRPC создание поста пользователем  {}", user_id);

        let post = self.blog_service.create_post(user_id, create_post_req.title, create_post_req.content)
            .await
            .map_err(|error| {
                match error {
                    crate::domain::BlogError::Validation(msg) => Status::invalid_argument(msg),
                    _ => Status::internal("Ошибка создания поста"),
                }
            })?;

        // => ProtoBuf Post
        let response = PostResponse {
            post: Some(Post {
                id: post.id,
                title: post.title,
                content: post.content,
                author_id: post.author_id,
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            }),
        };

        Ok(Response::new(response))
    }


    // Обновление
    async fn update_post(&self, request: Request<UpdatePostRequest>) -> Result<Response<PostResponse>, Status> 
    {
        let user_id = self.verify_auth(&request).await?;
        let update_post_req = request.into_inner();

        // Немного костыльно, чтобы пробросить до доменного update None или Some
        let new_title = if update_post_req.title.is_empty() {None } else { Some(update_post_req.title) };
        let new_content = if update_post_req.content.is_empty() { None } else { Some(update_post_req.content) };

        let post = self.blog_service.update_post(update_post_req.id, user_id, new_title, new_content)
            .await
            .map_err(|error| {
                match error {
                    // HTTP 404 и HTTP 403 
                    crate::domain::BlogError::NotFound(_) => Status::not_found("Пост не найден"),
                    crate::domain::BlogError::Forbidden => Status::permission_denied("Вы не автор"),
                    _ => Status::internal("Ошибка update'а"),
                }
            })?;

        // => ProtBuf Post
        let response = PostResponse {
            post: Some(Post {
                id: post.id,
                title: post.title,
                content: post.content,
                author_id: post.author_id,
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            }),
        };

        Ok(Response::new(response))
    }

    // Удаление
    async fn delete_post(&self, request: Request<DeletePostRequest>) -> Result<Response<DeletePostResponse>, Status> 
    {
        let user_id = self.verify_auth(&request).await?;
        let req = request.into_inner();

        self.blog_service.delete_post(req.id, user_id)
            .await
            .map_err(|error| {
                match error {
                    // HTTP 404 и HTTP 403 
                    crate::domain::BlogError::NotFound(_) => Status::not_found("Пост не найден"),
                    crate::domain::BlogError::Forbidden => Status::permission_denied("Вы не автор"),
                    _ => Status::internal("Ошибка удаления"),
                }
            })?;

        Ok(Response::new(DeletePostResponse { success: true }))
    }
}
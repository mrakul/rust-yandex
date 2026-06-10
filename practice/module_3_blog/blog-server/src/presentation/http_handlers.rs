use actix_web::{get, post, put, delete, web, HttpRequest, HttpResponse};
// use actix_web::HttpMessage;
use serde::{Deserialize};
use tracing::info;

use crate::application::{AuthService, BlogService};
use crate::data::{PostgresUserRepository, PostgresPostRepository};
use crate::domain::BlogError;
use crate::presentation::dto::*;
use crate::presentation::middleware::extract_authenticated_user;

/*** Public API ***/

// #[get("/health")]
async fn health() -> impl actix_web::Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        timestamp: chrono::Utc::now(),
    })
}

#[post("/auth/register")]
async fn register(
    auth: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<RegisterRequest>,
) -> Result<HttpResponse, BlogError> {
    let (user, token) = auth
        .register(
            payload.username.clone(),
            payload.email.clone(),
            payload.password.clone(),
        )
        .await?;

    info!(user_id = user.id, username = %user.username, "user registered");

    let response = AuthResponse {
        token,
        user: UserPublic::from(user),
    };

    Ok(HttpResponse::Created().json(response))
}

#[post("/auth/login")]
async fn login(
    auth: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<LoginRequest>,
) -> Result<HttpResponse, BlogError> {
    let (user, token) = auth
        .login(payload.username.clone(), payload.password.clone())
        .await?;

    info!(username = %user.username, "user logged in");

    let response = AuthResponse {
        token,
        user: UserPublic::from(user),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/posts")]
async fn list_posts(
    blog: web::Data<BlogService<PostgresPostRepository>>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, BlogError> {
    let limit = query.limit.unwrap_or(10).min(100).max(1);
    let offset = query.offset.unwrap_or(0);

    let (posts, total) = blog.list_posts(limit, offset).await?;

    let response = ListPostsResponse {
        posts: posts.into_iter().map(PostPublic::from).collect(),
        total,
        limit,
        offset,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/posts/{id}")]
async fn get_post(
    blog: web::Data<BlogService<PostgresPostRepository>>,
    path: web::Path<i64>,
) -> Result<HttpResponse, BlogError> {
    let post = blog.get_post(path.into_inner()).await?;
    
    Ok(HttpResponse::Ok().json(PostPublic::from(post)))
}

/*** Protected API, с авторизацией ***/

#[post("/posts")]
async fn create_post(
    req: HttpRequest,
    blog: web::Data<BlogService<PostgresPostRepository>>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, BlogError> {
    let user = extract_authenticated_user(&req)
        .ok_or(BlogError::Unauthorized)?;

    let post = blog
        .create_post(user.user_id, payload.title.clone(), payload.content.clone())
        .await?;

    info!(post_id = post.id, author_id = post.author_id, "Пост создан: ");

    Ok(HttpResponse::Created().json(PostPublic::from(post)))
}

#[put("/posts/{id}")]
async fn update_post(
    req: HttpRequest,
    blog: web::Data<BlogService<PostgresPostRepository>>,
    path: web::Path<i64>,
    payload: web::Json<UpdatePostRequest>,
) -> Result<HttpResponse, BlogError> {
    let user = extract_authenticated_user(&req)
        .ok_or(BlogError::Unauthorized)?;

    let post = blog
        .update_post(
            path.into_inner(),
            user.user_id,
            payload.title.clone(),
            payload.content.clone(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(PostPublic::from(post)))
}

#[delete("/posts/{id}")]
async fn delete_post(
    req: HttpRequest,
    blog: web::Data<BlogService<PostgresPostRepository>>,
    path: web::Path<i64>,
) -> Result<HttpResponse, BlogError> {
    let user = extract_authenticated_user(&req)
        .ok_or(BlogError::Unauthorized)?;

    blog.delete_post(path.into_inner(), user.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

// Пагинация
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Настройка public/protected
// Тут передаём &mut web::ServiceConfig, чтобы могло работать в .configure()
pub fn configure_public(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health))
    // Все через factory
    // cfg.service(health)
        .service(register)
        .service(login)
        .service(list_posts)
        .service(get_post);
}

pub fn configure_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(create_post)
        .service(update_post)
        .service(delete_post);
}
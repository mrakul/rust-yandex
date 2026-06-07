mod domain;
mod application;
mod data;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;


use application::{AuthService, BlogService};
use data::{PostgresUserRepository, PostgresPostRepository};
use infrastructure::{create_pool, run_migrations, init_logging, JwtService};
use presentation::{http_handlers, jwt_validator,
                   // gRPC части
                   grpc_service::BlogGrpcService, generated::blog_service_server::BlogServiceServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Логирование
    init_logging();

    // Загрузка .env файлика
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
         let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set (min 32 chars)");

    // Подключение к базе данных, миграции
    let pool = create_pool(&database_url)
        .await
        .expect("failed to connect to database");
        run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    // Репозитории и сервисы: авторизация, блок и JWT
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let post_repo = Arc::new(PostgresPostRepository::new(pool));

    let auth_service = AuthService::new(user_repo, jwt_secret.clone());
    let blog_service = BlogService::new(post_repo);
    let jwt_service = JwtService::new(jwt_secret);

    // CORS
    // let cors = Cors::default()
    //     .allow_any_origin()  // 🔥 For dev only; use .allowed_origin() in prod
    //     .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
    //     .allowed_headers(vec![
    //         actix_web::http::header::CONTENT_TYPE,
    //         actix_web::http::header::AUTHORIZATION,
    //     ])
    //     .supports_credentials()
    //     .max_age(3600);

    // JWT-middleware для Bearer'а (actix_web_httpauth)
    let jwt_middleware = HttpAuthentication::bearer(jwt_validator);

    // HTTP-сервер
    let http_server_addr = format!("{}:{}", 
        std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into()),
        std::env::var("PORT").unwrap_or_else(|_| "3000".into())
    );
    tracing::info!("🚀 HTTP-сервер запускается на http://{}", http_server_addr);

    // gRPC-сервер, сервисы, адрес
    let grpc_auth_service = auth_service.clone();
    let grpc_blog_service = blog_service.clone();
    let grpc_jwt_service = jwt_service.clone();
    
    let blog_grpc_service = BlogGrpcService::new(grpc_auth_service, grpc_blog_service, grpc_jwt_service);
    let grpc_server_addr = format!("{}:50051", 
        std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into())
    );

   let grpc_task = tonic::transport::Server::builder()
        .add_service(BlogServiceServer::new(blog_grpc_service))
        .serve(grpc_server_addr.parse().unwrap());

    // Запускаем через tokio::spawn
    tokio::spawn(async move {
        tracing::info!("🚀 gRPC-сервер запускается на http://{}", grpc_server_addr.clone());
        if let Err(error) = grpc_task.await {
            tracing::error!("gRPC ошибка: {}", error);
        }
    });

    // HTTP-сервер 
    HttpServer::new(move || {
        // Здесь не очень понял, в примерах в уроке тоже cors создаётся вовне и передаётся через .clone()
        // TODO: подумать, влияет ли
        let cors = Cors::default()
            .allow_any_origin()  // TODO: да, здесь лучше .allowed_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
            ])
            .supports_credentials()
            .max_age(3600);


        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            // Передаём сервисы в app_data
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(blog_service.clone()))
            .app_data(web::Data::new(jwt_service.clone()))
            .service(
                web::scope("/api")
                    // Публичный API: без авторизации
                    .configure(http_handlers::configure_public)
                    // Protected API: с middleware JWT
                    .service(
                        web::scope("")
                            .wrap(jwt_middleware.clone())
                            .configure(http_handlers::configure_protected)
                    )
            )
    })
    .bind(&http_server_addr)?
    .run()
    .await

}
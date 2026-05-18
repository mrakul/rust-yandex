mod domain;
mod application;
mod data;
mod presentation;
mod infrastructure;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{web, App, HttpServer};

use application::auth_service::AuthService;
use application::bank_service::BankService;
use data::account_repository::InMemoryAccountRepository;
use data::user_repository::InMemoryUserRepository;
use infrastructure::config::AppConfig;
use infrastructure::logging::init_logging;
use infrastructure::security::JwtKeys;
use presentation::handlers;
use presentation::middleware::{JwtAuthMiddleware, RequestIdMiddleware, TimingMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let config = AppConfig::from_env().expect("invalid configuration");

    let account_repo = Arc::new(InMemoryAccountRepository::default());
    let user_repo = Arc::new(InMemoryUserRepository::default());

    let bank_service = BankService::new(Arc::clone(&account_repo));
    let auth_service = AuthService::new(Arc::clone(&user_repo), JwtKeys::new(config.jwt_secret.clone()));

    let config_data = config.clone();

    HttpServer::new(move || {
        let cors = build_cors(&config_data);
        App::new()
            .wrap(Logger::default())
            .wrap(RequestIdMiddleware)
            .wrap(TimingMiddleware)
            .wrap(DefaultHeaders::new()
                .add(("X-Content-Type-Options", "nosniff"))
                .add(("Referrer-Policy", "no-referrer"))
                .add(("Permissions-Policy", "geolocation=()"))
                .add(("Cross-Origin-Opener-Policy", "same-origin")))
            .wrap(cors)
            .app_data(web::Data::new(bank_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .service(
                web::scope("/api")
                    .service(handlers::public::scope())
                    .service(
                        web::scope("")
                            .wrap(JwtAuthMiddleware::new(auth_service.keys().clone()))
                            .service(handlers::protected::scope())
                    )
            )
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}

fn build_cors(config: &AppConfig) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::AUTHORIZATION,
        ])
        .supports_credentials()
        .max_age(3600);

    for origin in &config.cors_origins {
        cors = cors.allowed_origin(origin);
    }

    cors
}


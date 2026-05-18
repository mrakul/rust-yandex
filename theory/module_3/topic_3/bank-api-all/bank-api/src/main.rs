mod presentation;
mod application;
mod domain;
mod data;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use application::bank_service::BankService;
use data::account_repository::InMemoryAccountRepository;
use presentation::handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Data: пока в памяти, используется Arc
    let repo = Arc::new(InMemoryAccountRepository::default());
    // BankService с передачей repo
    let service = BankService::new(repo);

    HttpServer::new(move || {
        // Разрешение только с определённых источников
        let cors = Cors::default()
            .allowed_origin("https://myapp.com")
            .allowed_origin("https://www.myapp.com")
            .allowed_origin("http://localhost:3000") 
            // Для разработки, в некоторых ситуациях бывает и для прода (бэкенд проксирует запросы в кор)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
            ])
            .expose_headers(vec!["X-Total-Count"]) // чтобы фронт видел заголовки ответа
            .supports_credentials() // Для cookies
            .max_age(3600);
       
        // # Запрос с разрешённого origin (должен пройти)
        // curl -H "Origin: https://myapp.com" \
        //      -H "Access-Control-Request-Method: POST" \
        //      -H "Access-Control-Request-Headers: Content-Type" \
        //      -X OPTIONS http://localhost:8080/api/accounts

        // # Запрос с неразрешённого origin (должен быть заблокирован)
        // curl -H "Origin: https://evil.com" \
        //      -X POST http://localhost:8080/api/accounts 

        App::new()
            // Middleware выполняются в обратном порядке при ответе!
            .wrap(Logger::default())             // 1. Логирование (внешнее кольцо)
            .wrap(cors)
            // .wrap(custom_middleware)          // 2. Кастомная логика
            // Передаём сервис с репозиторием
            .app_data(web::Data::new(service.clone()))
            // Конфигурируем фукнции с путями, указанными через макросы
            .configure(handlers::configure)     // 3. Handlers
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
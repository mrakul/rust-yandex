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
        let cors = Cors::permissive();
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            // Передаём сервис с репозиторием
            .app_data(web::Data::new(service.clone()))
            // Конфигурируем фукнции с путями, указанными через макросы
            .configure(handlers::configure)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
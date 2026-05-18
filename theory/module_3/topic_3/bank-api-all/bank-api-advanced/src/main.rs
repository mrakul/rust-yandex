mod domain;
mod application;
mod data;
mod presentation;
mod infrastructure;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use application::bank_service::BankService;
use data::account_repository::InMemoryAccountRepository;
use infrastructure::logging::init_logging;
use presentation::handlers;
use presentation::middleware::{RequestIdMiddleware, TimingMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let repo = Arc::new(InMemoryAccountRepository::default());
    let service = BankService::new(Arc::clone(&repo));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            // [MR] Добавлены MiddleWare, я их проверил в http-сервер'е
            .wrap(RequestIdMiddleware)
            .wrap(TimingMiddleware)
            .wrap(Cors::permissive())
            .app_data(web::Data::new(service.clone()))
            .configure(handlers::configure)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


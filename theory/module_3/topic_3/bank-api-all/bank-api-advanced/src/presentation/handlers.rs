use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder, Scope};
use chrono::Utc;
use tracing::info;

use crate::application::bank_service::BankService;
use crate::data::account_repository::InMemoryAccountRepository;
use crate::domain::BankError;
use crate::presentation::dto::{
    AccountResponse, AmountRequest, CreateAccountRequest, HealthResponse, TransferRequest,
};
use crate::presentation::middleware::RequestId;

fn request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<RequestId>()
        .map(|rid| rid.0.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        timestamp: Utc::now().to_rfc3339(),
    })
}

#[post("/accounts")]
async fn create_account(
    req: HttpRequest,
    service: web::Data<BankService<InMemoryAccountRepository>>,
    payload: web::Json<CreateAccountRequest>,
) -> Result<impl Responder, BankError> {
    let rid = request_id(&req);
    let account = service
        .create_account(payload.id, payload.initial)
        .await?;

    info!(
        request_id = %rid,
        account_id = account.id,
        initial = payload.initial,
        "account created"
    );

    Ok(HttpResponse::Created().json(AccountResponse::from(account)))
}

#[get("/accounts/{id}")]
async fn get_account(
    req: HttpRequest,
    service: web::Data<BankService<InMemoryAccountRepository>>,
    path: web::Path<u32>,
) -> Result<impl Responder, BankError> {
    let rid = request_id(&req);
    let account_id = path.into_inner();
    let account = service.get_account(account_id).await?;

    info!(
        request_id = %rid,
        account_id,
        "account fetched"
    );

    Ok(HttpResponse::Ok().json(AccountResponse::from(account)))
}

#[post("/accounts/{id}/deposit")]
async fn deposit(
    req: HttpRequest,
    service: web::Data<BankService<InMemoryAccountRepository>>,
    path: web::Path<u32>,
    payload: web::Json<AmountRequest>,
) -> Result<impl Responder, BankError> {
    let rid = request_id(&req);
    let account_id = path.into_inner();
    let amount = payload.amount;
    let account = service.deposit(account_id, amount).await?;

    info!(
        request_id = %rid,
        account_id,
        amount,
        "deposit completed"
    );

    Ok(HttpResponse::Ok().json(AccountResponse::from(account)))
}

#[post("/accounts/{id}/withdraw")]
async fn withdraw(
    req: HttpRequest,
    service: web::Data<BankService<InMemoryAccountRepository>>,
    path: web::Path<u32>,
    payload: web::Json<AmountRequest>,
) -> Result<impl Responder, BankError> {
    let rid = request_id(&req);
    let account_id = path.into_inner();
    let amount = payload.amount;
    let account = service.withdraw(account_id, amount).await?;

    info!(
        request_id = %rid,
        account_id,
        amount,
        "withdraw completed"
    );

    Ok(HttpResponse::Ok().json(AccountResponse::from(account)))
}

#[post("/transfers")]
async fn transfer(
    req: HttpRequest,
    service: web::Data<BankService<InMemoryAccountRepository>>,
    payload: web::Json<TransferRequest>,
) -> Result<impl Responder, BankError> {
    let rid = request_id(&req);
    service
        .transfer(payload.from, payload.to, payload.amount)
        .await?;

    info!(
        request_id = %rid,
        from = payload.from,
        to = payload.to,
        amount = payload.amount,
        "transfer completed"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "transferred"
    })))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    let scope: Scope = web::scope("/api")
        .service(health)
        .service(create_account)
        .service(get_account)
        .service(deposit)
        .service(withdraw)
        .service(transfer);

    cfg.service(scope);
}


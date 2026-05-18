use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;
use serde_json::json;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum DomainError {
    #[error("invalid amount: {0}")]
    InvalidAmount(String),
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("account not found")]
    AccountNotFound,
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum BankError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("insufficient funds for account {0}")]
    InsufficientFunds(u32),
    #[error("unauthorized")]
    Unauthorized,
    #[error("database error: {0}")]
    Database(String),
}

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ResponseError for BankError {
    fn status_code(&self) -> StatusCode {
        match self {
            BankError::Validation(_) => StatusCode::BAD_REQUEST,
            BankError::NotFound(_) => StatusCode::NOT_FOUND,
            BankError::InsufficientFunds(_) => StatusCode::BAD_REQUEST,
            BankError::Unauthorized => StatusCode::UNAUTHORIZED,
            BankError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = self.to_string();
        let details = match self {
            BankError::Validation(msg) => Some(json!({ "message": msg })),
            BankError::NotFound(resource) => Some(json!({ "resource": resource })),
            BankError::InsufficientFunds(account) => {
                Some(json!({ "account_id": account, "reason": "insufficient_funds" }))
            }
            BankError::Unauthorized => None,
            BankError::Database(msg) => Some(json!({ "message": msg })),
        };

        let payload = ErrorEnvelope {
            error: &message,
            details,
        };

        HttpResponse::build(self.status_code()).json(payload)
    }
}

impl From<DomainError> for BankError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::InvalidAmount(msg) => BankError::Validation(msg),
            DomainError::InsufficientFunds => BankError::InsufficientFunds(0),
            DomainError::AccountNotFound => BankError::NotFound("account".into()),
        }
    }
}


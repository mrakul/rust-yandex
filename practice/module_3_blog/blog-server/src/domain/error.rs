use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
// На примере error.rs из теории через thiserror
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    // Здесь пока есть неиспользуемые
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("user already exists: {0}")]
    UserAlreadyExists(String),
    #[error("user not found: {0}")]
    UserNotFound(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("post not found: {0}")]
    PostNotFound(i64),
    
    #[error("forbidden: user is not the author of the post")]
    Forbidden,
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum BlogError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("conflict: {0}")]   // HTTP 409 Conflict: уже зарегистрирован и другие
    Conflict(String),
    #[error("internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

// Коды возвратов
impl ResponseError for BlogError {
    fn status_code(&self) -> StatusCode {
        match self {
            BlogError::Validation(_) => StatusCode::BAD_REQUEST,
            BlogError::NotFound(_) => StatusCode::NOT_FOUND,
            BlogError::Unauthorized => StatusCode::UNAUTHORIZED,
            BlogError::Forbidden => StatusCode::FORBIDDEN,
            BlogError::Conflict(_) => StatusCode::CONFLICT,
            BlogError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = self.to_string();
        let details = match self {
            BlogError::Validation(msg) => Some(serde_json::json!({ "message": msg })),
            BlogError::NotFound(resource) => Some(serde_json::json!({ "resource": resource })),
            BlogError::Conflict(msg) => Some(serde_json::json!({ "message": msg })),
            _ => None,
        };
        let body = ErrorBody {
            error: &message,
            details,
        };
        HttpResponse::build(self.status_code()).json(body)
    }
}

// Соответствие доменных -> блогерских ошибок
impl From<DomainError> for BlogError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Validation(msg) => BlogError::Validation(msg),
            DomainError::UserAlreadyExists(_) => BlogError::Conflict("user already exists".into()),
            DomainError::UserNotFound(_) => BlogError::NotFound("user not found".into()),
            DomainError::InvalidCredentials => BlogError::Unauthorized,
            DomainError::PostNotFound(id) => BlogError::NotFound(format!("post {}", id)),
            DomainError::Forbidden => BlogError::Forbidden,
            DomainError::Internal(msg) => BlogError::Internal(msg),
        }
    }
}
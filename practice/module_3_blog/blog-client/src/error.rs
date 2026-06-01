use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlogClientError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    #[error("Post not found: {0}")]
    PostNotFound(i64),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Unexpected status code: {0}")]
    UnexpectedStatus(u16),
    
    // Пока возвращается для gRPC
    #[error("Unsupported transport: {0}")]
    UnsupportedTransport(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String)
}
pub mod middleware;
pub mod http_handlers;
// Заглушка для gRPC
// pub mod grpc_service;

pub mod dto;
pub use middleware::jwt_validator;
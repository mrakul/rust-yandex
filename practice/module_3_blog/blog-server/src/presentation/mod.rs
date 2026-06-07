pub mod middleware;
pub mod http_handlers;
// gRPC-сервис
pub mod grpc_service;

// Включаем сгенерированный код то build.rs, "namespace" - generated, хотя можно и в top-уровень включить
pub mod generated {
    // package blog из ProtoBuf
    tonic::include_proto!("blog");
}

pub mod dto;
pub use middleware::jwt_validator;
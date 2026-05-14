use actix_web::{web, App, HttpServer, HttpResponse, Result, FromRequest};

use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::collections::HashMap;  // TODO:
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing::info;

mod presentation;

// TODO: пока всё напихано в один файл


// 1. По умолчания: actix_web=info,http_server=debug (добавил другие модули), то есть cargo run запустит с такими значениями
// 2. Или глобальный фильтр: RUST_LOG=debug cargo run
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
            // Уровни логирования
            .unwrap_or_else(|_| 
                    "actix_web=debug,\
                     actix_server=debug,\
                     actix_http=debug,\
                     http_server=debug"
            .into())
            
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
        )
        .init();
    
    tracing::info!("Logging initialized");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    // Конфигурация через переменные окружения
    let config = Config {
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/bank".to_string()),
        redis_url: std::env::var("REDIS_URL").ok(),
        max_connections: std::env::var("MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10),
    };

    // Создаём instance AppState
    let app_state = web::Data::new(AppState {
        // В асинхронном коде используйте синхронизацию из tokio::sync — это предотвратит блокировку рантайма.
        // Вы храните в App State общий счётчик и пул БД. Доступ к счётчику на чтение требуется часто, а на запись — редко. Какой механизм синхронизации выбрать для счётчика?
        // tokio::sync::RwLock<i64>
        counter: RwLock::new(0),
        database_url: "postgres://localhost/bank".to_string(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Конфигурация
            .app_data(web::Data::new(config.clone()))
            
            // Добавляем тайминг и RequestId middleware
            .wrap(presentation::middleware::TimingMiddleware)      // измерение времени
            .wrap(presentation::middleware::RequestIdMiddleware)   // корреляционный ID

            .service(web::scope("/api")
                .route("/", web::get().to(index))
                .route("/health", web::get().to(health))
                .route("/users", web::post().to(create_user))
                .route("/users", web::get().to(get_users))
                .route("/users/{id}", web::get().to(get_user)) 
                .route("/increment", web::post().to(increment))
                .route("/counter", web::get().to(get_counter))
                .route("/ws/chat/{id}", web::get().to(chat_ws)) 
            )
        // web::scope облегчает организацию API по префиксам и версиям:

        // App::new()
        //     .service(web::scope("/api")
        //         .route("/users", web::get().to(get_users))
        //     ) 

        // А версионирование через разные области позволяет развивать API без поломки существующих клиентов:
        // App::new()
        //     .service(web::scope("/api/v1")
        //         .route("/users", web::get().to(get_users_v1))
        //     )
        //     .service(web::scope("/api/v2")
        //         .route("/users", web::get().to(get_users_v2))
        //     ) 
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 

// В реальных сервисах разные обработчики делят общие ресурсы: подключения к базе данных, кеш, конфигурацию. 
//  Для передачи этих ресурсов внутрь обработчиков используется AppState — глобальное состояние.
// Через web::Data вы аккуратно передаёте состояние и получаете к нему типобезопасный доступ
struct AppState {
    counter: RwLock<i32>,
    database_url: String,
}

// curl -X POST 127.0.0.1:8080/increment
async fn increment(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut counter = state.counter.write().await;
    *counter += 1;
    Ok(HttpResponse::Ok().json(*counter))
}

// curl 127.0.0.1:8080/counter
async fn get_counter(state: web::Data<AppState>) -> Result<HttpResponse> {
    let counter = state.counter.read().await;
    Ok(HttpResponse::Ok().json(*counter))
}

// Конфигурация
#[derive(Deserialize, Clone)]
struct Config {
    database_url: String,
    redis_url: Option<String>,
    max_connections: u32,
}

// Работоспособность
async fn health(config: web::Data<Config>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "redis_configured": config.redis_url.is_some(),
        "max_connections": config.max_connections
    })))
}


// Вы можете расширять приложение новыми маршрутами. Например, добавить точку проверки состояния:

// 127.0.0.1:8080/health
// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "ok",
//         "timestamp\n": chrono::Utc::now()
//     })))
// }


// curl http://localhost:8080/api/users/123
async fn get_user(path: web::Path<u32>) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    
    let user = find_user(user_id).await
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
    
    Ok(HttpResponse::Ok().json(format!("User ID: {}", user_id)))
}

async fn find_user(user_id: u32) ->  Option<User> {
    // Заглушка на первого пользователя
    if user_id == 1 {
        Some(User { id: 1, name: "Alice".into(), email: "alice@example.com".into() })
    } else {
        None
    }
}

// С проверкой ошибки
// async fn get_user(path: web::Path<u32>) -> Result<HttpResponse> {
//     let user_id = path.into_inner();
    
//     match find_user_by_id(user_id).await {
//         Some(user) => Ok(HttpResponse::Ok().json(user)),
//         None => Ok(HttpResponse::NotFound().json(serde_json::json!({
//             "error": "User not found"
//         }))),
//     }
// } 


// Для нескольких сегментов вы можете распаковать кортеж:
// Маршрут: /users/{user_id}/posts/{post_id} 
async fn get_user_post(path: web::Path<(u32, u32)>) -> Result<HttpResponse> {
    let (user_id, post_id) = path.into_inner();
    Ok(HttpResponse::Ok().json(format!("User: {}, Post: {}", user_id, post_id)))
}


// Query-параметры — фильтрация и пагинация. Строка после ? десериализуется в вашу структуру:

#[derive(Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

// Маршрут: /users?page=1&limit=20 
async fn get_users(query: web::Query<PaginationQuery>) -> Result<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    
    Ok(HttpResponse::Ok().json(format!(
        "Page: {}, Limit: {}", page, limit
    )))
}


// Body-параметры — JSON-данные. Тело POST/PUT-запросов удобно парсить в типизированные структуры:
#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Маршрут: POST /users 
async fn create_user(user: web::Json<CreateUserRequest>) -> Result<HttpResponse, AppError> {
    info!(email = %user.email, "Creating user");

    // Структурированное логирование

    // tracing::info!(
    //     user_id = 123,
    //     account_id = 456,
    //     amount = 1000,
    //     "Transfer completed"
    // );

    // TODO:
    let new_user = User {
        id: 1,
        name: user.name.clone(),
        email: user.email.clone(),
    };

    if user.email.is_empty() {
        return Err(AppError::Validation("Email cannot be empty".into()));
    }

    // TODO:
    // let new_user = save_user(&user).await?; // sqlx::Error автоматически конвертируется в AppError
    

    // Проверка с возвратом ошибки BadRequest
    // if user.name.is_empty() {
    //     return Ok(HttpResponse::BadRequest().json(serde_json::json!({
    //         "error": "Name cannot be empty"
    //     })));
    // }
    
    // match process_user(&user).await {
    //     Ok(u) => {
    //         info!(user_id = u.id, "User created successfully");
    //         Ok(HttpResponse::Created().json(u))
    //     }
    //     Err(e) => {
    //         tracing::error!(error = %e, "Failed to create user");
    //         Err(actix_web::error::ErrorInternalServerError(e))
    //     }
    // };

    // Обработка валидных данных...
    Ok(HttpResponse::Created().json(new_user))
} 

// async fn process_user(req: &CreateUserRequest) -> Result<User, String> {
//     debug!("Processing user data");
//     // ...

// }


// Комбинирование параметров. 
// Разные экстракторы могут использоваться одновременно. 
// Порядок аргументов обработчика значения не имеет — Actix-web сопоставит их по типам.

async fn update_user(
    path: web::Path<u32>,                    // user_id из URL
    query: web::Query<PaginationQuery>,      // page из query
    user: web::Json<CreateUserRequest>       // данные из body
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    
    Ok(HttpResponse::Ok().json(format!(
        "Updated user {} on page {}", user_id, page
    )))
}

// async fn index() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().body("Hello, World!\n"))
// }

async fn index() -> Result<HttpResponse> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>My API</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .endpoint { background: #f5f5f5; padding: 10px; margin: 10px 0; }
    </style>
</head>
<body>
    <h1>My API</h1>
    <p>Welcome to our API!</p>
        
        <div class="endpoint">
        <strong>GET /api/users</strong> - Get all users
        </div>
        <div class="endpoint">
        <strong>POST /api/users</strong> - Create user
    </div>
</body>
</html>
    "#;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html))
} 

// Для раздачи статики подходит actix_files::Files, а для сложных страниц — шаблонизаторы вроде Askama.


/*** Ошибки ***/
use actix_web::{error::ResponseError, http::StatusCode};
use thiserror::Error;

// 1. Короткий вариант

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation failed: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    // #[error("Database error: {0}")]
    // Database(#[from] sqlx::Error),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            // AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        HttpResponse::build(status).json(serde_json::json!({
            "error": self.to_string(),
            "status": status.as_u16(),
        }))
    }
}

// 2. Расширенный вариант

// #[derive(Debug, Error)]
// pub enum AppError {
//     #[error("Validation failed")]
//     Validation {
//         fields: Vec<String>,
//         message: String,
//     },
    
//     #[error("Unauthorized: {reason}")]
//     Unauthorized { reason: String },
// }

// #[derive(Serialize)]
// struct ErrorResponse {
//     error: String,
//     details: Option<serde_json::Value>,
// }

// impl ResponseError for AppError {
//     fn error_response(&self) -> HttpResponse {
//         let (status, details) = match self {
//             AppError::Validation { fields, message } => (
//                 StatusCode::BAD_REQUEST,
//                 Some(serde_json::json!({
//                     "fields": fields,
//                     "message": message,
//                 }))
//             ),
//             AppError::Unauthorized { reason } => (
//                 StatusCode::UNAUTHORIZED,
//                 Some(serde_json::json!({
//                     "reason": reason,
//                 }))
//             ),
//         };
        
//         HttpResponse::build(status).json(ErrorResponse {
//             error: self.to_string(),
//             details,
//         })
//     }
    
//     fn status_code(&self) -> StatusCode {
//         match self {
//             AppError::Validation { .. } => StatusCode::BAD_REQUEST,
//             AppError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
//         }
//     }
// } 


/*** Экстракторы ***/

// Экстрактор авторизованного пользователя

// use actix_web::{
//     dev::Payload,
//     error::ErrorUnauthorized,
//     web::{Data, FromRequest},
//     Error, HttpRequest,
// };

// use std::future::{ready, Ready};

// #[derive(Debug, Clone, Serialize)]
// pub struct AuthenticatedUser {
//     pub id: u32,
//     pub email: String,
// }

// impl FromRequest for AuthenticatedUser {
//     type Error = Error;
//     type Future = Ready<Result<Self, Self::Error>>;

//     fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
//         // Извлекаем токен из заголовка Authorization
//         let auth_header = req.headers().get("Authorization")
//             .and_then(|h| h.to_str().ok());
        
//         let token = match auth_header {
//             Some(header) if header.starts_with("Bearer ") => {
//                 &header[7..]
//             }
//             _ => {
//                 return ready(Err(ErrorUnauthorized("Missing or invalid Authorization header")))
//             }
//         };
        
//         // Получаем конфигурацию с секретом
//         let config = match req.app_data::<Data<AppConfig>>() {
//             Some(cfg) => cfg,
//             None => {
//                 return ready(Err(ErrorUnauthorized("Configuration not found")))
//             }
//         };
        
//         // Валидируем токен (пример валидации)
//         match validate_token(token, &config.secret_key) {
//             Ok(claims) => {
//                 match claims.user_id.parse::<u32>() {
//                     Ok(user_id) => ready(Ok(AuthenticatedUser {
//                         id: user_id,
//                         email: claims.email,
//                     })),
//                     Err(_) => ready(Err(ErrorUnauthorized("Invalid user ID in token"))),
//                 }
//             }
//             Err(_) => ready(Err(ErrorUnauthorized("Invalid or expired token"))),
//         }
//     }
// } 

// Пример использования в handler'ах:

// async fn get_profile(user: AuthenticatedUser) -> HttpResponse {
//     HttpResponse::Ok().json(serde_json::json!({
//         "user_id": user.id,
//         "email": user.email,
//     }))
// }

// // Регистрация
// App::new()
//     .app_data(web::Data::new(config))
//     .route("/profile", web::get().to(get_profile)) 


/*** Web-сокет ***/

// WebSocket обеспечивает постоянное двунаправленное соединение между клиентом и сервером. 
// Полезно для real-time уведомлений, чатов, игр, live-обновлений данных.

// Для двунаправленного real-time соединения в actix-web используется актор с StreamHandler, где сообщения парсятся и обрабатываются по мере поступления. Для надёжности соединения нужны пинги и тайм-ауты, а также контроль максимального размера кадров и обратного давления, чтобы почтовый ящик актора не раздувался. Если из других частей системы нужно отправлять клиенту событие, это делается через адрес актора, не ломая инкапсуляцию.
// Пинги/понги и лимит размера кадров защищают сервер от висящих соединений и DoS на почтовый ящик актора.

use actix::prelude::*;
use actix_web::{Error, HttpRequest};
use actix_web_actors::ws;


#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

pub struct ChatWebSocket {
    id: usize,
}

impl Actor for ChatWebSocket {
    type Context = ws::WebsocketContext<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::info!("WebSocket client {} connected", self.id);
    }
    
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        tracing::info!("WebSocket client {} disconnected", self.id);
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWebSocket {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(bytes)) => ctx.pong(&bytes),
            Ok(ws::Message::Text(text)) => {
                // Парсим JSON-сообщение от клиента
                match serde_json::from_str::<ChatMessage>(&text) {
                    Ok(chat_msg) => {
                        tracing::info!("Received: {:?}", chat_msg);
                        // Отправляем ответ
                        let response = ChatResponse {
                            from: self.id,
                            message: format!("Echo: {}", chat_msg.message),
                        };
                        ctx.text(serde_json::to_string(&response).unwrap());
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse message: {}", e);
                        ctx.text(r#"{"error": "Invalid message format"}"#);
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

#[derive(Deserialize, Debug)]
struct ChatMessage {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    from: usize,
    message: String,
}

async fn chat_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let id = req.match_info().get("id")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    
    ws::start(ChatWebSocket { id }, &req, stream)
}

// Отправка сообщений из других акторов:
// use actix::prelude::*;

// impl Handler<WsMessage> for ChatWebSocket {
//     type Result = ();
    
//     fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
//         ctx.text(msg.0);
//     }
// }

// // Отправка сообщения из другого актора
// let addr: Addr<ChatWebSocket> = /* ... */;
// addr.do_send(WsMessage("Hello from server".to_string())); 
use actix_web::{web, App, HttpServer, HttpResponse, Result};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::collections::HashMap;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {

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
            .service(web::scope("/api")
                .route("/", web::get().to(index))
                .route("/health", web::get().to(health))
                .route("/users", web::get().to(get_users))
                .route("/users/{id}", web::get().to(get_user)) 
                .route("/increment", web::post().to(increment))
                .route("/counter", web::get().to(get_counter))
            )
        // web::scope облегчает организацию API по префиксам и версиям:

        // App::new()
        //     .service(web::scope("/api")
        //         .route("/users", web::get().to(get_users))
        //         .route("/users", web::post().to(create_user))
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

// Вы можете расширять приложение новыми маршрутами. Например, добавить точку проверки состояния:

// 127.0.0.1:8080/health
// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "ok",
//         "timestamp\n": chrono::Utc::now()
//     })))
// }

async fn get_user(path: web::Path<u32>) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    Ok(HttpResponse::Ok().json(format!("User ID: {}", user_id)))
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
async fn create_user(user: web::Json<CreateUserRequest>) -> Result<HttpResponse> {

    let new_user = User {
        id: 1,
        name: user.name.clone(),
        email: user.email.clone(),
    };

    // Проверка с возвратом ошибки BadRequest
    if user.name.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Name cannot be empty"
        })));
    }
    
    // Обработка валидных данных...
    Ok(HttpResponse::Created().json(new_user))
} 


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
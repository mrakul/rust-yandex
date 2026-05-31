use reqwest::Client;
use std::time::Duration;

// В реальных приложениях часто нужно обращаться к внешним сервисам: платёжным системам, email-сервисам, геокодингу, курсам валют и т. д. 
// В Rust для этого используют reqwest — асинхронный HTTP-клиент.

// ✅ Клиент с настройками и rustls (рекомендуется)
pub fn create_http_client() -> Client {
    Client::builder()
        .user_agent("bank-api/1.0")
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to create HTTP client")
}

// ✅ Переиспользование клиента (рекомендуется)
// Создайте один клиент и передавайте его через Arc или web::Data

/*** GET-запросы ***/

// ✅ GET с query-параметрами
async fn get_rate_for_currency(
    client: &Client,
    from: &str,
    to: &str,
) -> Result<f64, reqwest::Error> {
    let url = format!("https://api.exchangerate.host/convert?from={}&to={}", from, to);
    
    let response = client
        .get(&url)
        .send()
        .await?;
    
    let data: serde_json::Value = response.json().await?;
    Ok(data["result"].as_f64().unwrap_or(0.0))
}

// ✅ GET с заголовками
async fn get_with_auth(client: &Client, api_key: &str) -> Result<String, reqwest::Error> {
    let response = client
        .get("https://api.example.com/data")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("X-API-Key", api_key)
        .send()
        .await?;
    
    let text = response.text().await?;
    Ok(text)
}

// POST-запросы:

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct PaymentRequest {
    amount: i64,
    currency: String,
    recipient: String,
}

#[derive(Deserialize)]
struct PaymentResponse {
    transaction_id: String,
    status: String,
}

// ✅ POST с JSON
async fn create_payment(
    client: &Client,
    request: PaymentRequest,
) -> Result<PaymentResponse, reqwest::Error> {
    let response = client
        .post("https://api.payment.com/payments")
        .json(&request)
        .send()
        .await?;
    
    let payment: PaymentResponse = response.json().await?;
    Ok(payment)
}

// ✅ POST с form data
async fn send_email(
    client: &Client,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), reqwest::Error> {
    let form = [
        ("to", to),
        ("subject", subject),
        ("body", body),
    ];
    
    client
        .post("https://api.email.com/send")
        .form(&form)
        .send()
        .await?;
    
    Ok(())
} 

// Обработка ошибок:

use reqwest::Client;
use thiserror::Error;

#[derive(Error, Debug)]
enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Timeout")]
    Timeout,
}

// ✅ Правильная обработка ошибок
async fn call_external_api(client: &Client, url: &str) -> Result<String, ApiError> {
    let response = client
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                ApiError::Timeout
            } else {
                ApiError::Http(e)
            }
        })?;
    
    // Проверяем статус ответа
    if !response.status().is_success() {
        return Err(ApiError::Api(format!(
            "API returned status: {}",
            response.status()
        )));
    }
    
    let text = response.text().await?;
    Ok(text)
} 

// Интеграция с actix-web:

use actix_web::{web, App, HttpServer, HttpResponse, Result};
use reqwest::Client;
use std::sync::Arc;

// Функция создания клиента (из предыдущего примера)
fn create_http_client() -> Client {
    Client::builder()
        .user_agent("bank-api/1.0")
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("Failed to create HTTP client")
}

async fn get_exchange_rate_handler(
    client: web::Data<Arc<Client>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let from = query.get("from").ok_or("Missing 'from' parameter")?;
    let to = query.get("to").ok_or("Missing 'to' parameter")?;
    
    match get_rate_for_currency(client.get_ref(), from, to).await {
        Ok(rate) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "from": from,
            "to": to,
            "rate": rate
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch rate: {}", e)
        }))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Arc::new(create_http_client());
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(client.clone()))
            .route("/exchange", web::get().to(get_exchange_rate_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 

// Retry-логика. Для надёжности часто нужна retry-логика при временных сбоях:

use std::time::Duration;
use tokio::time::sleep;

async fn call_with_retry(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<String, reqwest::Error> {
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response.text().await?);
                }
                // Если статус не успешный, пробуем ещё раз
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
        
        if attempt < max_retries {
            // Экспоненциальная задержка: 1s, 2s, 4s
            let delay = Duration::from_secs(2_u64.pow(attempt));
            sleep(delay).await;
        }
    }
    
    Err(last_error.unwrap())
}
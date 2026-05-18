use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Account;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub id: u32,
    pub initial: i64,
}

#[derive(Debug, Deserialize)]
pub struct AmountRequest {
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from: u32,
    pub to: u32,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: u32,
    pub balance: i64,
    pub owner_id: Uuid,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            balance: account.balance,
            owner_id: account.owner_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: DateTime<Utc>,
}


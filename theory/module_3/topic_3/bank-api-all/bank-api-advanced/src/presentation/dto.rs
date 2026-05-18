use serde::{Deserialize, Serialize};

use crate::domain::Account;

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub id: u32,
    #[serde(default)]
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
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            balance: account.balance,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: String,
}


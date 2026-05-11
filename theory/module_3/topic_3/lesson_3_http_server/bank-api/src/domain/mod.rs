use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: u32,
    pub balance: i64,
}

impl Account {
    pub fn new(id: u32, initial_balance: i64) -> Result<Self, DomainError> {
        if initial_balance < 0 {
            return Err(DomainError::InvalidAmount(
                "Initial balance must be non-negative".into(),
            ));
        }
        Ok(Self {
            id,
            balance: initial_balance,
        })
    }

    pub fn deposit(&mut self, amount: Amount) {
        self.balance += amount.0;
    }

    pub fn withdraw(&mut self, amount: Amount) -> Result<(), DomainError> {
        if self.balance < amount.0 {
            return Err(DomainError::InsufficientFunds);
        }
        self.balance -= amount.0;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Amount(pub i64);

impl Amount {
    pub fn new(value: i64) -> Result<Self, DomainError> {
        if value <= 0 {
            return Err(DomainError::InvalidAmount(
                "Amount must be positive".into(),
            ));
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub from: u32,
    pub to: u32,
    pub amount: Amount,
}

impl Transfer {
    pub fn new(from: u32, to: u32, raw_amount: i64) -> Result<Self, DomainError> {
        if from == to {
            return Err(DomainError::InvalidAmount(
                "Cannot transfer to the same account".into(),
            ));
        }
        Ok(Self {
            from,
            to,
            amount: Amount::new(raw_amount)?,
        })
    }
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid amount: {0}")]
    InvalidAmount(String),
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("account not found")]
    AccountNotFound,
}


pub mod error;
pub mod user;

use serde::{Deserialize, Serialize};

pub use error::{BankError, DomainError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: u32,
    pub balance: i64,
    pub owner_id: uuid::Uuid,
}

impl Account {
    pub fn new(id: u32, owner_id: uuid::Uuid, initial_balance: i64) -> Result<Self, DomainError> {
        if initial_balance < 0 {
            return Err(DomainError::Validation("initial balance must be non-negative".into()));
        }
        Ok(Self {
            id,
            balance: initial_balance,
            owner_id,
        })
    }

    pub fn deposit(&mut self, amount: Amount) {
        self.balance += amount.0;
    }

    pub fn withdraw(&mut self, amount: Amount) -> Result<(), DomainError> {
        if self.balance < amount.0 {
            return Err(DomainError::InsufficientFunds(self.id));
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
            return Err(DomainError::Validation("amount must be positive".into()));
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
            return Err(DomainError::Validation(
                "source and destination accounts must differ".into(),
            ));
        }
        Ok(Self {
            from,
            to,
            amount: Amount::new(raw_amount)?,
        })
    }
}


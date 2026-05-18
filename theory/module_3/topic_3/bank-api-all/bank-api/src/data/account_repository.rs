use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::{Account, DomainError};

// Тут именно trait - интерфейс для работы с внутренностями: в памяти, в БД, другие.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn create(&self, account: Account) -> Result<(), DomainError>;
    async fn get(&self, id: u32) -> Result<Option<Account>, DomainError>;
    async fn upsert(&self, account: Account) -> Result<(), DomainError>;
}

// Структура для хранения в памяти
#[derive(Default, Clone)]
pub struct InMemoryAccountRepository {
    accounts: Arc<RwLock<HashMap<u32, Account>>>,
}

// Реализация интерфейса для хранения в памяти
#[async_trait]
impl AccountRepository for InMemoryAccountRepository {
    async fn create(&self, account: Account) -> Result<(), DomainError> {
        let mut accounts = self.accounts.write().await;
        if accounts.contains_key(&account.id) {
            return Err(DomainError::InvalidAmount("Account already exists".into()));
        }
        accounts.insert(account.id, account);
        Ok(())
    }

    async fn get(&self, id: u32) -> Result<Option<Account>, DomainError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.get(&id).cloned())
    }

    async fn upsert(&self, account: Account) -> Result<(), DomainError> {
        let mut accounts = self.accounts.write().await;
        accounts.insert(account.id, account);
        Ok(())
    }
}

impl InMemoryAccountRepository {
    #[allow(dead_code)]
    pub fn snapshot(&self) -> Arc<RwLock<HashMap<u32, Account>>> {
        Arc::clone(&self.accounts)
    }
}


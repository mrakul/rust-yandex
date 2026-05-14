use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::{Account, DomainError};

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn create(&self, account: Account) -> Result<(), DomainError>;
    async fn get(&self, id: u32) -> Result<Option<Account>, DomainError>;
    async fn upsert(&self, account: Account) -> Result<(), DomainError>;
}

#[derive(Default, Clone)]
pub struct InMemoryAccountRepository {
    accounts: Arc<RwLock<HashMap<u32, Account>>>,
}

#[async_trait]
impl AccountRepository for InMemoryAccountRepository {
    async fn create(&self, account: Account) -> Result<(), DomainError> {
        let mut accounts = self.accounts.write().await;
        if accounts.contains_key(&account.id) {
            return Err(DomainError::InvalidAmount("account already exists".into()));
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


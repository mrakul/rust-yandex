use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{user::User, DomainError};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> Result<User, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
}

#[derive(Default, Clone)]
pub struct InMemoryUserRepository {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    emails: Arc<RwLock<HashMap<String, Uuid>>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, user: User) -> Result<User, DomainError> {
        let mut users = self.users.write().await;
        let mut emails = self.emails.write().await;

        if emails.contains_key(&user.email) {
            return Err(DomainError::Validation("email already registered".into()));
        }

        emails.insert(user.email.clone(), user.id);
        users.insert(user.id, user.clone());
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let emails = self.emails.read().await;
        let users = self.users.read().await;

        if let Some(id) = emails.get(email) {
            Ok(users.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let users = self.users.read().await;
        Ok(users.get(&id).cloned())
    }
}


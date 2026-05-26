use std::sync::Arc;
use tracing::instrument;
// use uuid::Uuid;

use crate::domain::{user::User, BlogError, DomainError};
use crate::infrastructure::jwt::{hash_password, verify_password, JwtService};
use crate::data::user_repository::UserRepository;

// Аналогично, основано на примере из теории

#[derive(Clone)]
pub struct AuthService<R: UserRepository + 'static> {
    repo: Arc<R>,
    jwt: JwtService,
}

impl<R> AuthService<R>
where
    R: UserRepository + 'static,
{
    pub fn new(repo: Arc<R>, jwt_secret: String) -> Self {
        Self {
            repo,
            jwt: JwtService::new(jwt_secret),
        }
    }

    // Регистрация пользователя:
    // При регистрации возвращаю токен сразу для логина, пароль не логируем
    #[instrument(skip(self, password))]
    pub async fn register(&self, username: String, email: String, password: String) -> Result<(User, String), BlogError> {
        // Validate input
        if username.len() < 3 || username.len() > 50 {
            return Err(BlogError::Validation("Имя пользователя должно быть от 3 до 50 символов".into()));
        }
        if !email.contains('@') {
            return Err(BlogError::Validation("Неверный формат e-mail".into()));
        }
        if password.len() < 8 {
            return Err(BlogError::Validation("Длина пароля - не менее 8 символов".into()));
        }

        // Уже зарегистрирован - по имени или по e-mail
        if self.repo.find_by_username(&username).await?.is_some() {
            return Err(DomainError::UserAlreadyExists(username).into());
        }

        if self.repo.find_by_email(&email).await?.is_some() {
            return Err(DomainError::UserAlreadyExists(email).into());
        }

        // Хеш пароля и создание пользователя
        let password_hash = hash_password(&password)
            .map_err(|e| BlogError::Internal(format!("Не получилось взять хеш пароля: {}", e)))?;
        
        let mut user = User::new(username.clone(), email.clone(), password_hash);
        user = self.repo.create(user).await.map_err(BlogError::from)?;

        // JWT-token - при регистрации тоже
        let token = self.jwt.generate_token(user.id, username)
            .map_err(|e| BlogError::Internal(format!("Ошибка генерации токена (internal): {}", e)))?;

        Ok((user, token))
    }

    /// Логин с проверкой пароля 
    #[instrument(skip(self, password))]
    pub async fn login(&self, username: String, password: String) -> Result<(User, String), BlogError> {
        let user = self.repo.find_by_username(&username).await?
            .ok_or(BlogError::Unauthorized)?;

        // Неверный пароль или ошибка при верификации => Unauthorized
        if !verify_password(&password, &user.password_hash)
            .map_err(|_| BlogError::Unauthorized)? 
        {
            return Err(BlogError::Unauthorized);
        }

        let token = self.jwt.generate_token(user.id, user.username.clone())
            .map_err(|e| BlogError::Internal(format!("Ошибка генерации токена (internal): {}", e)))?;

        Ok((user, token))
    }

    // Пока добавляю до gRPC
    #[allow(dead_code)]
    pub async fn get_user(&self, user_id: i64) -> Result<User, BlogError> {
        self.repo.find_by_id(user_id).await?
            .ok_or_else(|| BlogError::NotFound(format!("Пользователь {}", user_id)))
    }

    // pub fn jwt_service(&self) -> &JwtService {
    //     &self.jwt
    // }
}
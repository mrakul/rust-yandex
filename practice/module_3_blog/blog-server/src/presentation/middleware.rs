use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::infrastructure::jwt::JwtService;

// Авторизованный пользователь, которого кладём в ServiceRequest
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    // Тут оперирую только user_id, закомментил для искоренения Warning'а
    // pub username: String
}

pub async fn jwt_validator(req: ServiceRequest,
                           credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)>
{
   let jwt_service = match req.app_data::<actix_web::web::Data<JwtService>>() {
        Some(data) => data.get_ref().clone(),
        None => {
            // Можно добавить tracing::warn!/error!
            return Err((ErrorUnauthorized("JwtService не подключён (отсутствует в app_data)"), req));
        }
    };

    // Используем actix_web_httpauth
    let claims = match jwt_service.verify_token(credentials.token()) {
        Ok(claims) => claims,
        Err(_) => {
            return Err((ErrorUnauthorized("Неверный или просроченный токен"), req));
        }
    };


    let user_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return Err((ErrorUnauthorized("Неверный user_id"), req));
        }
    };

    let user = AuthenticatedUser {
        user_id,
        // username: claims.username,
    };

    req.extensions_mut().insert(user);

    Ok(req)
}

// Вспомогательная функция для хендлеров
pub fn extract_authenticated_user(req: &actix_web::HttpRequest) -> Option<AuthenticatedUser> {
    req.extensions().get::<AuthenticatedUser>().cloned()
}
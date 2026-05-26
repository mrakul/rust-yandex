use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;  // ← Correct import!

use crate::infrastructure::jwt::JwtService;

/// Authenticated user extracted from JWT and inserted into request extensions.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

/// JWT validator for actix-web-httpauth bearer middleware.
/// Signature MUST match: (ServiceRequest, BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)>
pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,  // ← Correct type from extractors::bearer
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // Extract JwtService from app_data
    let jwt_service = match req.app_data::<actix_web::web::Data<JwtService>>() {
        Some(data) => data.get_ref().clone(),
        None => {
            return Err((
                ErrorUnauthorized("JwtService not configured"),
                req,
            ));
        }
    };

    // Verify the token
    let claims = match jwt_service.verify_token(credentials.token()) {
        Ok(claims) => claims,
        Err(_) => {
            return Err((ErrorUnauthorized("invalid or expired token"), req));
        }
    };

    // Parse user_id from claims
    let user_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return Err((ErrorUnauthorized("invalid user_id in token"), req));
        }
    };

    // Insert authenticated user into request extensions
    let user = AuthenticatedUser {
        user_id,
        username: claims.username,
    };
    req.extensions_mut().insert(user);

    Ok(req)
}

/// Helper to extract authenticated user from request extensions (for handlers).
pub fn extract_authenticated_user(
    req: &actix_web::HttpRequest,
) -> Option<AuthenticatedUser> {
    req.extensions().get::<AuthenticatedUser>().cloned()
}
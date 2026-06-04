// Вынес отдельно DTO

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthResponse {
    pub token: String,
    user: UserDto
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserDto {
    id: i64,
    username: String,
    email: String,
}
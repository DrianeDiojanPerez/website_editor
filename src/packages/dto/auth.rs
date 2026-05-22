use serde::{Deserialize, Serialize};

use crate::packages::dto::user::UserDto;

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshDto {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutDto {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPairDto {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,         // access-token lifetime, seconds
    pub refresh_expires_in: i64, // refresh-token lifetime, seconds
}

#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub user: UserDto,
    pub token: TokenPairDto,
}

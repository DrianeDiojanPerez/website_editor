use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::packages::model::user::User;

// Password is never returned to clients.
#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub is_system: bool,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            is_system: u.is_system != 0,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NewUserDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserDto {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

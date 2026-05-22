use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::packages::model::user::User;

// Password is never returned to clients.
#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub is_system: bool,
    pub must_change_password: bool,
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
            must_change_password: u.must_change_password != 0,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewUserDto {
    #[validate(length(min = 5, message = "must be at least 5 characters"))]
    pub username: String,

    #[validate(email(message = "must be a valid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub password: String,
}

// Optional fields — `validator` only runs the check if the field is `Some`.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(length(min = 5, message = "must be at least 5 characters"))]
    pub username: Option<String>,

    #[validate(email(message = "must be a valid email address"))]
    pub email: Option<String>,

    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordDto {
    #[validate(length(min = 1, message = "is required"))]
    pub current_password: String,

    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForcePasswordResetDto {
    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub temporary_password: String,
}

// -------------------------------------------------------------------------
// Unit tests for the validation derives. These don't touch a database, a
// network socket, or anything async — they exercise the `validator` macro
// output directly. Run with `cargo test`.
// -------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn valid_new_user() -> NewUserDto {
        NewUserDto {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "longenough".to_string(),
        }
    }

    #[test]
    fn new_user_dto_accepts_a_valid_payload() {
        assert!(valid_new_user().validate().is_ok());
    }

    #[test]
    fn new_user_dto_rejects_short_username() {
        let dto = NewUserDto { username: "ab".to_string(), ..valid_new_user() };
        let errs = dto.validate().unwrap_err();
        assert!(errs.field_errors().contains_key("username"));
    }

    #[test]
    fn new_user_dto_rejects_malformed_email() {
        let dto = NewUserDto { email: "not-an-email".to_string(), ..valid_new_user() };
        let errs = dto.validate().unwrap_err();
        assert!(errs.field_errors().contains_key("email"));
    }

    #[test]
    fn new_user_dto_rejects_short_password() {
        let dto = NewUserDto { password: "tiny".to_string(), ..valid_new_user() };
        let errs = dto.validate().unwrap_err();
        assert!(errs.field_errors().contains_key("password"));
    }

    #[test]
    fn new_user_dto_reports_every_failing_field_at_once() {
        let dto = NewUserDto {
            username: "ab".to_string(),
            email: "nope".to_string(),
            password: "x".to_string(),
        };
        let errs = dto.validate().unwrap_err();
        let fields = errs.field_errors();
        assert!(fields.contains_key("username"));
        assert!(fields.contains_key("email"));
        assert!(fields.contains_key("password"));
    }

    #[test]
    fn update_user_dto_skips_validation_for_none_fields() {
        // None values aren't checked — only Some(...) is validated.
        let dto = UpdateUserDto { username: None, email: None, password: None };
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn update_user_dto_rejects_short_password_when_present() {
        let dto = UpdateUserDto {
            username: None,
            email: None,
            password: Some("x".to_string()),
        };
        let errs = dto.validate().unwrap_err();
        assert!(errs.field_errors().contains_key("password"));
    }
}

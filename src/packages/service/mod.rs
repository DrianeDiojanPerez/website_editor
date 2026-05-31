pub mod auth;
pub mod project;
pub mod project_member;
pub mod project_version;
pub mod user;

use std::collections::HashMap;

use thiserror::Error;

use crate::packages::codes;
use crate::packages::repository::RepoError;

#[derive(Debug, Error)]
pub struct ServiceError {
    pub message: String,
    pub code: i32,
    // Populated only by validation errors. Maps `"field_name"` → `["msg", …]`.
    pub fields: Option<HashMap<String, Vec<String>>>,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ServiceError {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self { message: message.into(), code, fields: None }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;

pub fn internal_error() -> ServiceError {
    ServiceError::new("internal server error", codes::INTERNAL_ERROR)
}

pub fn err_not_found(resource: &str) -> ServiceError {
    ServiceError::new(format!("{resource} not found"), codes::ERR_RESOURCE_NOT_FOUND)
}

pub fn err_validation(msg: impl Into<String>) -> ServiceError {
    ServiceError::new(msg, codes::ERR_VALIDATION)
}

pub fn err_unauthorized(msg: impl Into<String>) -> ServiceError {
    ServiceError::new(msg, codes::ERR_UNAUTHORIZED)
}

pub fn err_invalid_credentials() -> ServiceError {
    ServiceError::new("invalid credentials", codes::ERR_INVALID_CREDENTIALS)
}

pub fn err_invalid_refresh_token() -> ServiceError {
    ServiceError::new(
        "invalid or expired refresh token",
        codes::ERR_INVALID_REFRESH_TOKEN,
    )
}

// Centralised message formatter — Rust equivalent of Go's `GetErrorMsg(fe)`.
// Looks at the rule code that failed (`length`, `email`, `range`, ...) and
// the parameters the rule was configured with (e.g. `min = 5`) and renders a
// human-readable message. Add new cases here when you adopt new rules.
//
// DTOs no longer need `#[validate(length(min = 5, message = "..."))]` — they
// can just say `#[validate(length(min = 5))]` and let this function format it.
// An explicit `message = "..."` on the attribute still wins if you ever need
// to override.
fn message_for(err: &validator::ValidationError) -> String {
    if let Some(m) = &err.message {
        return m.to_string();
    }

    // Pull a stringified param value out of the error's params map.
    let p = |key: &str| -> Option<String> {
        err.params.get(key).map(|v| match v {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        })
    };

    match err.code.as_ref() {
        "required" => "This field is required".to_string(),

        "length" => {
            if let Some(eq) = p("equal") {
                format!("Must be exactly {eq} characters")
            } else if let (Some(min), Some(max)) = (p("min"), p("max")) {
                format!("Must be between {min} and {max} characters")
            } else if let Some(min) = p("min") {
                format!("Must be at least {min} characters")
            } else if let Some(max) = p("max") {
                format!("Must be at most {max} characters")
            } else {
                "Invalid length".to_string()
            }
        }

        "range" => {
            if let (Some(min), Some(max)) = (p("min"), p("max")) {
                format!("Must be between {min} and {max}")
            } else if let Some(min) = p("min") {
                format!("Must be at least {min}")
            } else if let Some(max) = p("max") {
                format!("Must be at most {max}")
            } else {
                "Out of range".to_string()
            }
        }

        "email" => "Invalid email format".to_string(),
        "url" => "Invalid URL".to_string(),
        "regex" => "Invalid format".to_string(),
        "non_control_character" => "Must not contain control characters".to_string(),
        "credit_card" => "Invalid credit card number".to_string(),
        "phone" => "Invalid phone number".to_string(),

        "contains" => match p("pattern") {
            Some(pat) => format!("Must contain `{pat}`"),
            None => "Missing required content".to_string(),
        },
        "does_not_contain" => match p("pattern") {
            Some(pat) => format!("Must not contain `{pat}`"),
            None => "Contains forbidden content".to_string(),
        },

        "must_match" => match p("other") {
            Some(other) => format!("Must match `{other}`"),
            None => "Fields do not match".to_string(),
        },

        // Fallback: any custom validator that did NOT supply a message will
        // render as its code (e.g. "invalid_role"). Add explicit cases above
        // for known custom codes, or set `.message = ...` on the
        // ValidationError inside the custom function.
        other => other.to_string(),
    }
}

// Used by the ValidatedJson extractor to translate `validator::ValidationErrors`
// into a `ServiceError` that carries a `{field: [messages]}` map.
pub fn err_from_validation(errors: validator::ValidationErrors) -> ServiceError {
    let mut fields: HashMap<String, Vec<String>> = HashMap::new();
    for (field, errs) in errors.field_errors() {
        let msgs: Vec<String> = errs.iter().map(message_for).collect();
        fields.insert(field.to_string(), msgs);
    }
    ServiceError {
        message: "validation failed".to_string(),
        code: codes::ERR_VALIDATION,
        fields: Some(fields),
    }
}

impl From<RepoError> for ServiceError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::NotFound => err_not_found("resource"),
            RepoError::Sqlx(e) => {
                tracing::error!(error = ?e, "repository sqlx error");
                internal_error()
            }
        }
    }
}

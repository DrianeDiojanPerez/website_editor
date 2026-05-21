pub mod project;
pub mod project_member;
pub mod project_version;
pub mod user;

use thiserror::Error;

use crate::packages::codes;
use crate::packages::repository::RepoError;

#[derive(Debug, Error)]
pub struct ServiceError {
    pub message: String,
    pub code: i32,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ServiceError {
    pub fn new(message: impl Into<String>, code: i32) -> Self {
        Self { message: message.into(), code }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;

// Canonical service-layer errors — mirrors `service.NewError(...)` in the Go service.
pub fn internal_error() -> ServiceError {
    ServiceError::new("internal server error", codes::INTERNAL_ERROR)
}

pub fn err_not_found(resource: &str) -> ServiceError {
    ServiceError::new(format!("{resource} not found"), codes::ERR_RESOURCE_NOT_FOUND)
}

pub fn err_validation(msg: impl Into<String>) -> ServiceError {
    ServiceError::new(msg, codes::ERR_VALIDATION)
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

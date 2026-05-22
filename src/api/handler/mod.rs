pub mod auth;
pub mod health;
pub mod project;
pub mod project_member;
pub mod project_version;
pub mod user;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::packages::codes;
use crate::packages::dto::response::Response as ApiResponse;
use crate::packages::lib::jwt::TokenManager;
use crate::packages::repository::Store;
use crate::packages::service::auth::{new_auth_service, AuthService};
use crate::packages::service::project::{new_project_service, ProjectService};
use crate::packages::service::project_member::{
    new_project_member_service, ProjectMemberService,
};
use crate::packages::service::project_version::{
    new_project_version_service, ProjectVersionService,
};
use crate::packages::service::user::{new_user_service, UserService};
use crate::packages::service::ServiceError;

// Mirrors `Handler` in cmd/api/handler/handler.go — one field per service.
// `token_manager` is here so the JWT middleware (which uses State<Arc<Handler>>)
// can verify access tokens.
pub struct Handler {
    pub token_manager: Arc<TokenManager>,
    pub auth_service: Arc<dyn AuthService>,
    pub user_service: Arc<dyn UserService>,
    pub project_service: Arc<dyn ProjectService>,
    pub project_member_service: Arc<dyn ProjectMemberService>,
    pub project_version_service: Arc<dyn ProjectVersionService>,
}

pub fn configure_handlers(
    store: Arc<dyn Store>,
    token_manager: Arc<TokenManager>,
) -> Arc<Handler> {
    let user_service = new_user_service(store.clone());
    let auth_service = new_auth_service(store.clone(), user_service.clone(), token_manager.clone());
    Arc::new(Handler {
        auth_service,
        user_service,
        project_service: new_project_service(store.clone()),
        project_member_service: new_project_member_service(store.clone()),
        project_version_service: new_project_version_service(store),
        token_manager,
    })
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = match self.code {
            c if c == codes::ERR_RESOURCE_NOT_FOUND => StatusCode::NOT_FOUND,
            c if c == codes::ERR_VALIDATION => StatusCode::BAD_REQUEST,
            c if c == codes::ERR_UNAUTHORIZED
                || c == codes::ERR_INVALID_CREDENTIALS
                || c == codes::ERR_INVALID_REFRESH_TOKEN => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body: ApiResponse<()> = ApiResponse::err(self.code, self.message);
        (status, Json(body)).into_response()
    }
}

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
use crate::packages::repository::Store;
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
pub struct Handler {
    pub user_service: Arc<dyn UserService>,
    pub project_service: Arc<dyn ProjectService>,
    pub project_member_service: Arc<dyn ProjectMemberService>,
    pub project_version_service: Arc<dyn ProjectVersionService>,
}

// Equivalent of Go's `ConfigureHandlers` — wires every service from the store.
pub fn configure_handlers(store: Arc<dyn Store>) -> Arc<Handler> {
    Arc::new(Handler {
        user_service: new_user_service(store.clone()),
        project_service: new_project_service(store.clone()),
        project_member_service: new_project_member_service(store.clone()),
        project_version_service: new_project_version_service(store),
    })
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = match self.code {
            c if c == codes::ERR_RESOURCE_NOT_FOUND => StatusCode::NOT_FOUND,
            c if c == codes::ERR_VALIDATION => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body: ApiResponse<()> = ApiResponse::err(self.code, self.message);
        (status, Json(body)).into_response()
    }
}

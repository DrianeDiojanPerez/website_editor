pub mod item;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::packages::codes;
use crate::packages::dto::response::Response as ApiResponse;
use crate::packages::repository::Store;
use crate::packages::service::item::{new_item_service, ItemService};
use crate::packages::service::ServiceError;

// Mirrors `Handler` in cmd/api/handler/handler.go — one field per service.
pub struct Handler {
    pub item_service: Arc<dyn ItemService>,
}

impl Handler {
    pub fn new(item_service: Arc<dyn ItemService>) -> Self {
        Self { item_service }
    }
}

// Equivalent of Go's `ConfigureHandlers` — wires every service from the store.
pub fn configure_handlers(store: Arc<dyn Store>) -> Arc<Handler> {
    let item_service = new_item_service(store);
    Arc::new(Handler::new(item_service))
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

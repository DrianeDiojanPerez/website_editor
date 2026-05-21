use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

use super::handler::{item as item_handler, Handler};

pub fn router(handler: Arc<Handler>) -> Router {
    let v1 = Router::new()
        .route(
            "/items",
            get(item_handler::get_items).post(item_handler::create_item),
        )
        .route(
            "/items/:id",
            get(item_handler::get_item)
                .patch(item_handler::update_item)
                .delete(item_handler::delete_item),
        );

    Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api/v1", v1)
        .with_state(handler)
        .layer(TraceLayer::new_for_http())
}

use axum::http::header;
use axum::response::{IntoResponse, Response};

// Both files are embedded into the binary at compile time. No disk read at
// request time, and the binary can be moved anywhere.
const SCALAR_HTML: &str = include_str!("../../../docs/scalar.html");
const OPENAPI_YAML: &str = include_str!("../../../docs/openapi.yaml");

#[tracing::instrument]
pub async fn scalar() -> Response {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        SCALAR_HTML,
    )
        .into_response()
}

#[tracing::instrument]
pub async fn openapi_yaml() -> Response {
    (
        [(header::CONTENT_TYPE, "application/yaml; charset=utf-8")],
        OPENAPI_YAML,
    )
        .into_response()
}

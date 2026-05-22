use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;

use crate::api::handler::Handler;
use crate::packages::lib::jwt::Claims;
use crate::packages::service::{err_unauthorized, ServiceError};

// Extracts and verifies a Bearer access token from the Authorization header,
// then attaches the decoded Claims as a request extension for downstream
// handlers to read via `Extension<Claims>`.
pub async fn require_jwt(
    State(handler): State<Arc<Handler>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ServiceError> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| err_unauthorized("missing authorization header"))?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| err_unauthorized("authorization header must be `Bearer <token>`"))?;

    let claims: Claims = handler
        .token_manager
        .decode_access(token)
        .map_err(|_| err_unauthorized("invalid or expired token"))?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

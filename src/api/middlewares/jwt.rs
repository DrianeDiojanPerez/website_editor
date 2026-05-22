use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;

use crate::api::handler::Handler;
use crate::packages::codes;
use crate::packages::lib::jwt::Claims;
use crate::packages::service::{err_unauthorized, ServiceError};

// Verifies a `Authorization: Bearer <token>` header and attaches the decoded
// `Claims` as a request extension so downstream code can read it with
// `Extension<Claims>`.
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

// Blocks every request while the authenticated user is in the
// "must change password" state. The only endpoint that should bypass this
// is `/auth/change-password` itself.
//
// Apply *after* `require_jwt` so the Claims extension is already populated.
pub async fn require_password_set(
    req: Request<Body>,
    next: Next,
) -> Result<Response, ServiceError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| err_unauthorized("missing authenticated session"))?;

    if claims.must_change_password {
        return Err(ServiceError::new(
            "password change required before continuing",
            codes::ERR_PASSWORD_CHANGE_REQUIRED,
        ));
    }
    Ok(next.run(req).await)
}

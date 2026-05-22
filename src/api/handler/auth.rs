use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use axum::Extension;

use crate::api::extractors::ValidatedJson;
use crate::packages::dto::auth::{AuthResponseDto, LoginDto, LogoutDto, RefreshDto};
use crate::packages::dto::response::Response;
use crate::packages::dto::user::{ChangePasswordDto, NewUserDto, UserDto};
use crate::packages::lib::jwt::Claims;
use crate::packages::service::ServiceError;

use super::Handler;

#[tracing::instrument(skip(h, dto))]
pub async fn signup(
    State(h): State<Arc<Handler>>,
    ValidatedJson(dto): ValidatedJson<NewUserDto>,
) -> Result<(StatusCode, Json<Response<AuthResponseDto>>), ServiceError> {
    let r = h.auth_service.signup(dto).await?;
    Ok((StatusCode::CREATED, Json(Response::ok(r))))
}

#[tracing::instrument(skip(h, dto))]
pub async fn login(
    State(h): State<Arc<Handler>>,
    ValidatedJson(dto): ValidatedJson<LoginDto>,
) -> Result<Json<Response<AuthResponseDto>>, ServiceError> {
    Ok(Json(Response::ok(h.auth_service.login(dto).await?)))
}

#[tracing::instrument(skip(h, dto))]
pub async fn refresh(
    State(h): State<Arc<Handler>>,
    ValidatedJson(dto): ValidatedJson<RefreshDto>,
) -> Result<Json<Response<AuthResponseDto>>, ServiceError> {
    Ok(Json(Response::ok(h.auth_service.refresh(dto).await?)))
}

#[tracing::instrument(skip(h, dto))]
pub async fn logout(
    State(h): State<Arc<Handler>>,
    ValidatedJson(dto): ValidatedJson<LogoutDto>,
) -> Result<StatusCode, ServiceError> {
    h.auth_service.logout(dto).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Authenticated user changes their own password. Requires the JWT but NOT
// the password-set middleware — this is the one route a "must change password"
// user is allowed to hit.
#[tracing::instrument(skip(h, dto, claims))]
pub async fn change_password(
    State(h): State<Arc<Handler>>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(dto): ValidatedJson<ChangePasswordDto>,
) -> Result<Json<Response<UserDto>>, ServiceError> {
    let user = h.auth_service.change_password(claims.sub, dto).await?;
    Ok(Json(Response::ok(user)))
}

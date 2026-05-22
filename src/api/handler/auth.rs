use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::packages::dto::auth::{AuthResponseDto, LoginDto, LogoutDto, RefreshDto};
use crate::packages::dto::response::Response;
use crate::packages::dto::user::NewUserDto;
use crate::packages::service::ServiceError;

use super::Handler;

#[tracing::instrument(skip(h, dto))]
pub async fn signup(
    State(h): State<Arc<Handler>>,
    Json(dto): Json<NewUserDto>,
) -> Result<(StatusCode, Json<Response<AuthResponseDto>>), ServiceError> {
    let r = h.auth_service.signup(dto).await?;
    Ok((StatusCode::CREATED, Json(Response::ok(r))))
}

#[tracing::instrument(skip(h, dto))]
pub async fn login(
    State(h): State<Arc<Handler>>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<Response<AuthResponseDto>>, ServiceError> {
    Ok(Json(Response::ok(h.auth_service.login(dto).await?)))
}

#[tracing::instrument(skip(h, dto))]
pub async fn refresh(
    State(h): State<Arc<Handler>>,
    Json(dto): Json<RefreshDto>,
) -> Result<Json<Response<AuthResponseDto>>, ServiceError> {
    Ok(Json(Response::ok(h.auth_service.refresh(dto).await?)))
}

#[tracing::instrument(skip(h, dto))]
pub async fn logout(
    State(h): State<Arc<Handler>>,
    Json(dto): Json<LogoutDto>,
) -> Result<StatusCode, ServiceError> {
    h.auth_service.logout(dto).await?;
    Ok(StatusCode::NO_CONTENT)
}

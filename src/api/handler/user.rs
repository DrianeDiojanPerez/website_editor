use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::extractors::ValidatedJson;
use crate::packages::dto::response::Response;
use crate::packages::dto::user::{
    ForcePasswordResetDto, NewUserDto, UpdateUserDto, UserDto,
};
use crate::packages::service::ServiceError;

use super::Handler;

#[tracing::instrument(skip(h))]
pub async fn get_users(
    State(h): State<Arc<Handler>>,
) -> Result<Json<Response<Vec<UserDto>>>, ServiceError> {
    Ok(Json(Response::ok(h.user_service.list().await?)))
}

#[tracing::instrument(skip(h))]
pub async fn get_user(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
) -> Result<Json<Response<UserDto>>, ServiceError> {
    Ok(Json(Response::ok(h.user_service.get(id).await?)))
}

// Admin-style create: forces the new user to change their password on first use.
// (Self-signup goes through /auth/signup which does NOT force a reset.)
#[tracing::instrument(skip(h, dto))]
pub async fn create_user(
    State(h): State<Arc<Handler>>,
    ValidatedJson(dto): ValidatedJson<NewUserDto>,
) -> Result<(StatusCode, Json<Response<UserDto>>), ServiceError> {
    let user = h.user_service.create(dto, true).await?;
    Ok((StatusCode::CREATED, Json(Response::ok(user))))
}

#[tracing::instrument(skip(h, dto))]
pub async fn update_user(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
    ValidatedJson(dto): ValidatedJson<UpdateUserDto>,
) -> Result<Json<Response<UserDto>>, ServiceError> {
    Ok(Json(Response::ok(h.user_service.update(id, dto).await?)))
}

#[tracing::instrument(skip(h))]
pub async fn delete_user(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ServiceError> {
    h.user_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Admin sets a temporary password on a user and flips must_change_password.
// TODO: authorize this to admins only (no role system yet — any authenticated
// user can currently call it).
#[tracing::instrument(skip(h, dto))]
pub async fn force_password_reset(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
    ValidatedJson(dto): ValidatedJson<ForcePasswordResetDto>,
) -> Result<Json<Response<UserDto>>, ServiceError> {
    let user = h
        .user_service
        .force_password_reset(id, dto.temporary_password)
        .await?;
    Ok(Json(Response::ok(user)))
}

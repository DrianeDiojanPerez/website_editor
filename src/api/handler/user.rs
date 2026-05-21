use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::packages::dto::response::Response;
use crate::packages::dto::user::{NewUserDto, UpdateUserDto, UserDto};
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

#[tracing::instrument(skip(h, dto))]
pub async fn create_user(
    State(h): State<Arc<Handler>>,
    Json(dto): Json<NewUserDto>,
) -> Result<(StatusCode, Json<Response<UserDto>>), ServiceError> {
    let user = h.user_service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(Response::ok(user))))
}

#[tracing::instrument(skip(h, dto))]
pub async fn update_user(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateUserDto>,
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

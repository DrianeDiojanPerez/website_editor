use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::packages::dto::item::{ItemDto, NewItemDto, UpdateItemDto};
use crate::packages::dto::response::Response;
use crate::packages::service::ServiceError;

use super::Handler;

#[tracing::instrument(skip(h))]
pub async fn get_items(
    State(h): State<Arc<Handler>>,
) -> Result<Json<Response<Vec<ItemDto>>>, ServiceError> {
    let items = h.item_service.list().await?;
    Ok(Json(Response::ok(items)))
}

#[tracing::instrument(skip(h))]
pub async fn get_item(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
) -> Result<Json<Response<ItemDto>>, ServiceError> {
    let item = h.item_service.get(id).await?;
    Ok(Json(Response::ok(item)))
}

#[tracing::instrument(skip(h, dto))]
pub async fn create_item(
    State(h): State<Arc<Handler>>,
    Json(dto): Json<NewItemDto>,
) -> Result<(StatusCode, Json<Response<ItemDto>>), ServiceError> {
    let item = h.item_service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(Response::ok(item))))
}

#[tracing::instrument(skip(h, dto))]
pub async fn update_item(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateItemDto>,
) -> Result<Json<Response<ItemDto>>, ServiceError> {
    let item = h.item_service.update(id, dto).await?;
    Ok(Json(Response::ok(item)))
}

#[tracing::instrument(skip(h))]
pub async fn delete_item(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ServiceError> {
    h.item_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

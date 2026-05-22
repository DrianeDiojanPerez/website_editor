use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::extractors::ValidatedJson;
use crate::packages::dto::project::{NewProjectDto, ProjectDto, UpdateProjectDto};
use crate::packages::dto::response::Response;
use crate::packages::service::ServiceError;

use super::Handler;

#[tracing::instrument(skip(h))]
pub async fn get_projects(
    State(h): State<Arc<Handler>>,
) -> Result<Json<Response<Vec<ProjectDto>>>, ServiceError> {
    Ok(Json(Response::ok(h.project_service.list().await?)))
}

#[tracing::instrument(skip(h))]
pub async fn get_project(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
) -> Result<Json<Response<ProjectDto>>, ServiceError> {
    Ok(Json(Response::ok(h.project_service.get(id).await?)))
}

#[tracing::instrument(skip(h, dto))]
pub async fn create_project(
    State(h): State<Arc<Handler>>,
    ValidatedJson(dto): ValidatedJson<NewProjectDto>,
) -> Result<(StatusCode, Json<Response<ProjectDto>>), ServiceError> {
    let p = h.project_service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(Response::ok(p))))
}

#[tracing::instrument(skip(h, dto))]
pub async fn update_project(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
    ValidatedJson(dto): ValidatedJson<UpdateProjectDto>,
) -> Result<Json<Response<ProjectDto>>, ServiceError> {
    Ok(Json(Response::ok(
        h.project_service.update(id, dto).await?,
    )))
}

#[tracing::instrument(skip(h))]
pub async fn delete_project(
    State(h): State<Arc<Handler>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ServiceError> {
    h.project_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

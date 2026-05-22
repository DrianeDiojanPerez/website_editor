use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::extractors::ValidatedJson;
use crate::packages::dto::project_version::{NewProjectVersionDto, ProjectVersionDto};
use crate::packages::dto::response::Response;
use crate::packages::service::ServiceError;

use super::Handler;

#[tracing::instrument(skip(h))]
pub async fn list_project_versions(
    State(h): State<Arc<Handler>>,
    Path(project_id): Path<i64>,
) -> Result<Json<Response<Vec<ProjectVersionDto>>>, ServiceError> {
    Ok(Json(Response::ok(
        h.project_version_service
            .list_by_project(project_id)
            .await?,
    )))
}

#[tracing::instrument(skip(h))]
pub async fn get_project_version(
    State(h): State<Arc<Handler>>,
    Path((_project_id, id)): Path<(i64, i64)>,
) -> Result<Json<Response<ProjectVersionDto>>, ServiceError> {
    Ok(Json(Response::ok(
        h.project_version_service.get(id).await?,
    )))
}

#[tracing::instrument(skip(h, dto))]
pub async fn snapshot_project_version(
    State(h): State<Arc<Handler>>,
    Path(project_id): Path<i64>,
    ValidatedJson(dto): ValidatedJson<NewProjectVersionDto>,
) -> Result<(StatusCode, Json<Response<ProjectVersionDto>>), ServiceError> {
    let v = h
        .project_version_service
        .snapshot(project_id, dto)
        .await?;
    Ok((StatusCode::CREATED, Json(Response::ok(v))))
}

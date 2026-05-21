use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::packages::dto::project_member::{
    AttachMemberDto, ProjectMemberDto, UpdateMemberDto,
};
use crate::packages::dto::response::Response;
use crate::packages::service::ServiceError;

use super::Handler;

#[tracing::instrument(skip(h))]
pub async fn list_project_members(
    State(h): State<Arc<Handler>>,
    Path(project_id): Path<i64>,
) -> Result<Json<Response<Vec<ProjectMemberDto>>>, ServiceError> {
    Ok(Json(Response::ok(
        h.project_member_service.list_by_project(project_id).await?,
    )))
}

#[tracing::instrument(skip(h, dto))]
pub async fn attach_project_member(
    State(h): State<Arc<Handler>>,
    Path(project_id): Path<i64>,
    Json(dto): Json<AttachMemberDto>,
) -> Result<(StatusCode, Json<Response<ProjectMemberDto>>), ServiceError> {
    let m = h.project_member_service.attach(project_id, dto).await?;
    Ok((StatusCode::CREATED, Json(Response::ok(m))))
}

#[tracing::instrument(skip(h, dto))]
pub async fn update_project_member(
    State(h): State<Arc<Handler>>,
    Path((_project_id, id)): Path<(i64, i64)>,
    Json(dto): Json<UpdateMemberDto>,
) -> Result<Json<Response<ProjectMemberDto>>, ServiceError> {
    Ok(Json(Response::ok(
        h.project_member_service.update_role(id, dto).await?,
    )))
}

#[tracing::instrument(skip(h))]
pub async fn detach_project_member(
    State(h): State<Arc<Handler>>,
    Path((_project_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode, ServiceError> {
    h.project_member_service.detach(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

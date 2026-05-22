use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use crate::packages::model::project_version::ProjectVersion;

#[derive(Debug, Serialize)]
pub struct ProjectVersionDto {
    pub id: i64,
    pub project_id: i64,
    pub version_number: i64,
    pub object_snapshot: Value,
    pub created_by: Option<i64>,
    pub created_at: NaiveDateTime,
}

impl From<ProjectVersion> for ProjectVersionDto {
    fn from(v: ProjectVersion) -> Self {
        Self {
            id: v.id,
            project_id: v.project_id,
            version_number: v.version_number,
            object_snapshot: v.object_snapshot.0,
            created_by: v.created_by,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewProjectVersionDto {
    pub object_snapshot: Value,
    pub created_by: Option<i64>,
}

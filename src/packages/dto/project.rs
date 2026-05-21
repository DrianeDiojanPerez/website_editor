use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::packages::model::project::Project;

#[derive(Debug, Serialize)]
pub struct ProjectDto {
    pub id: i64,
    pub name: String,
    pub current_version: i64,
    pub object_data: Value,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Project> for ProjectDto {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            name: p.name,
            current_version: p.current_version,
            object_data: p.object_data.0,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NewProjectDto {
    pub name: String,
    #[serde(default)]
    pub object_data: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectDto {
    pub name: Option<String>,
    pub object_data: Option<Value>,
}

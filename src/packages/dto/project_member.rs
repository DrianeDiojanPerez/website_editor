use serde::{Deserialize, Serialize};

use crate::packages::model::project_member::ProjectMember;

#[derive(Debug, Serialize)]
pub struct ProjectMemberDto {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub role: String,
}

impl From<ProjectMember> for ProjectMemberDto {
    fn from(m: ProjectMember) -> Self {
        Self {
            id: m.id,
            project_id: m.project_id,
            user_id: m.user_id,
            role: m.role,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AttachMemberDto {
    pub user_id: i64,
    #[serde(default = "default_role")]
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberDto {
    pub role: String,
}

fn default_role() -> String {
    "viewer".to_string()
}

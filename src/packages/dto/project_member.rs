use serde::{Deserialize, Serialize};
use validator::Validate;

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

fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    if role == "editor" || role == "viewer" {
        Ok(())
    } else {
        // Custom validators report a `code` and an optional message.
        let mut err = validator::ValidationError::new("invalid_role");
        err.message = Some("must be `editor` or `viewer`".into());
        Err(err)
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct AttachMemberDto {
    #[validate(range(min = 1, message = "must be a positive id"))]
    pub user_id: i64,

    #[serde(default = "default_role")]
    #[validate(custom(function = "validate_role"))]
    pub role: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMemberDto {
    #[validate(custom(function = "validate_role"))]
    pub role: String,
}

fn default_role() -> String {
    "viewer".to_string()
}

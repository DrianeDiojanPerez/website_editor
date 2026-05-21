use chrono::NaiveDateTime;
use sqlx::types::Json;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProjectVersion {
    pub id: i64,
    pub project_id: i64,
    pub version_number: i64,
    pub object_snapshot: Json<serde_json::Value>,
    pub created_by: Option<i64>,
    pub created_at: NaiveDateTime,
}

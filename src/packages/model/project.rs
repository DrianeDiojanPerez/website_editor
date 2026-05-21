use chrono::NaiveDateTime;
use sqlx::types::Json;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub current_version: i64,
    pub object_data: Json<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

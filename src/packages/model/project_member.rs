#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProjectMember {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub role: String,
}

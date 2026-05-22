use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub is_system: i64,
    pub must_change_password: i64,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

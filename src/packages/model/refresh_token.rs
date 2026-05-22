use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub revoked: i64,
    pub created_at: NaiveDateTime,
}

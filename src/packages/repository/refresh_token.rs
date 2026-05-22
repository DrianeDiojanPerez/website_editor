use async_trait::async_trait;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;

use crate::packages::model::refresh_token::RefreshToken;
use crate::packages::repository::{RepoError, RepoResult};

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn create(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: NaiveDateTime,
    ) -> RepoResult<RefreshToken>;
    async fn find_active(&self, token_hash: &str) -> RepoResult<RefreshToken>;
    async fn revoke(&self, id: i64) -> RepoResult<()>;
    async fn revoke_all_for_user(&self, user_id: i64) -> RepoResult<()>;
}

pub struct SqliteRefreshTokenRepository {
    db: SqlitePool,
}

impl SqliteRefreshTokenRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

const COLS: &str = "id, user_id, token_hash, expires_at, revoked, created_at";

#[async_trait]
impl RefreshTokenRepository for SqliteRefreshTokenRepository {
    #[tracing::instrument(skip(self, token_hash))]
    async fn create(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: NaiveDateTime,
    ) -> RepoResult<RefreshToken> {
        let row = sqlx::query_as::<_, RefreshToken>(&format!(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) \
             VALUES (?, ?, ?) RETURNING {COLS}"
        ))
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    #[tracing::instrument(skip(self, token_hash))]
    async fn find_active(&self, token_hash: &str) -> RepoResult<RefreshToken> {
        sqlx::query_as::<_, RefreshToken>(&format!(
            "SELECT {COLS} FROM refresh_tokens \
             WHERE token_hash = ? AND revoked = 0 AND expires_at > datetime('now')"
        ))
        .bind(token_hash)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn revoke(&self, id: i64) -> RepoResult<()> {
        sqlx::query("UPDATE refresh_tokens SET revoked = 1 WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn revoke_all_for_user(&self, user_id: i64) -> RepoResult<()> {
        sqlx::query("UPDATE refresh_tokens SET revoked = 1 WHERE user_id = ? AND revoked = 0")
            .bind(user_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

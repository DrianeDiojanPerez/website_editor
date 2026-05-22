use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::packages::model::user::User;
use crate::packages::repository::{RepoError, RepoResult};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn list(&self) -> RepoResult<Vec<User>>;
    async fn get(&self, id: i64) -> RepoResult<User>;
    async fn get_by_email(&self, email: &str) -> RepoResult<User>;
    async fn create(
        &self,
        username: &str,
        email: &str,
        password: &str,
        must_change_password: bool,
    ) -> RepoResult<User>;
    async fn update(
        &self,
        id: i64,
        username: &str,
        email: &str,
        password: &str,
    ) -> RepoResult<User>;
    async fn delete(&self, id: i64) -> RepoResult<()>;

    // Admin-driven password reset — sets a (hashed) temp password and flips
    // the must_change_password flag.
    async fn set_password_and_force_reset(
        &self,
        id: i64,
        hashed_password: &str,
    ) -> RepoResult<User>;

    // Self-service password change — sets a new password and clears the
    // must_change_password flag in one statement.
    async fn change_password(&self, id: i64, hashed_password: &str) -> RepoResult<User>;
}

pub struct SqliteUserRepository {
    db: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

const COLS: &str = "id, username, email, password, is_system, must_change_password, \
                    last_login_at, created_at, updated_at";

#[async_trait]
impl UserRepository for SqliteUserRepository {
    #[tracing::instrument(skip(self))]
    async fn list(&self) -> RepoResult<Vec<User>> {
        let rows = sqlx::query_as::<_, User>(&format!(
            "SELECT {COLS} FROM users ORDER BY id DESC"
        ))
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> RepoResult<User> {
        sqlx::query_as::<_, User>(&format!("SELECT {COLS} FROM users WHERE id = ?"))
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_email(&self, email: &str) -> RepoResult<User> {
        sqlx::query_as::<_, User>(&format!("SELECT {COLS} FROM users WHERE email = ?"))
            .bind(email)
            .fetch_optional(&self.db)
            .await?
            .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self, password))]
    async fn create(
        &self,
        username: &str,
        email: &str,
        password: &str,
        must_change_password: bool,
    ) -> RepoResult<User> {
        let row = sqlx::query_as::<_, User>(&format!(
            "INSERT INTO users (username, email, password, must_change_password) \
             VALUES (?, ?, ?, ?) RETURNING {COLS}"
        ))
        .bind(username)
        .bind(email)
        .bind(password)
        .bind(if must_change_password { 1 } else { 0 })
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    #[tracing::instrument(skip(self, password))]
    async fn update(
        &self,
        id: i64,
        username: &str,
        email: &str,
        password: &str,
    ) -> RepoResult<User> {
        sqlx::query_as::<_, User>(&format!(
            "UPDATE users SET username = ?, email = ?, password = ?, \
                              updated_at = datetime('now') \
             WHERE id = ? \
             RETURNING {COLS}"
        ))
        .bind(username)
        .bind(email)
        .bind(password)
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: i64) -> RepoResult<()> {
        // System users cannot be deleted.
        let result = sqlx::query("DELETE FROM users WHERE id = ? AND is_system = 0")
            .bind(id)
            .execute(&self.db)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, hashed_password))]
    async fn set_password_and_force_reset(
        &self,
        id: i64,
        hashed_password: &str,
    ) -> RepoResult<User> {
        sqlx::query_as::<_, User>(&format!(
            "UPDATE users SET password = ?, must_change_password = 1, \
                              updated_at = datetime('now') \
             WHERE id = ? \
             RETURNING {COLS}"
        ))
        .bind(hashed_password)
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self, hashed_password))]
    async fn change_password(&self, id: i64, hashed_password: &str) -> RepoResult<User> {
        sqlx::query_as::<_, User>(&format!(
            "UPDATE users SET password = ?, must_change_password = 0, \
                              updated_at = datetime('now') \
             WHERE id = ? \
             RETURNING {COLS}"
        ))
        .bind(hashed_password)
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }
}

use async_trait::async_trait;
use serde_json::Value;
use sqlx::types::Json;
use sqlx::SqlitePool;

use crate::packages::model::project::Project;
use crate::packages::repository::{RepoError, RepoResult};

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn list(&self) -> RepoResult<Vec<Project>>;
    async fn get(&self, id: i64) -> RepoResult<Project>;
    async fn create(&self, name: &str, object_data: &Value) -> RepoResult<Project>;
    async fn update(
        &self,
        id: i64,
        name: &str,
        object_data: &Value,
    ) -> RepoResult<Project>;
    async fn bump_version(&self, id: i64) -> RepoResult<Project>;
    async fn delete(&self, id: i64) -> RepoResult<()>;
}

pub struct SqliteProjectRepository {
    db: SqlitePool,
}

impl SqliteProjectRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

const COLS: &str =
    "id, name, current_version, object_data, created_at, updated_at";

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    #[tracing::instrument(skip(self))]
    async fn list(&self) -> RepoResult<Vec<Project>> {
        let rows = sqlx::query_as::<_, Project>(&format!(
            "SELECT {COLS} FROM projects ORDER BY id DESC"
        ))
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> RepoResult<Project> {
        sqlx::query_as::<_, Project>(&format!("SELECT {COLS} FROM projects WHERE id = ?"))
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self, object_data))]
    async fn create(&self, name: &str, object_data: &Value) -> RepoResult<Project> {
        let row = sqlx::query_as::<_, Project>(&format!(
            "INSERT INTO projects (name, object_data) VALUES (?, ?) RETURNING {COLS}"
        ))
        .bind(name)
        .bind(Json(object_data))
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    #[tracing::instrument(skip(self, object_data))]
    async fn update(
        &self,
        id: i64,
        name: &str,
        object_data: &Value,
    ) -> RepoResult<Project> {
        sqlx::query_as::<_, Project>(&format!(
            "UPDATE projects SET name = ?, object_data = ?, updated_at = datetime('now') \
             WHERE id = ? \
             RETURNING {COLS}"
        ))
        .bind(name)
        .bind(Json(object_data))
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn bump_version(&self, id: i64) -> RepoResult<Project> {
        sqlx::query_as::<_, Project>(&format!(
            "UPDATE projects SET current_version = current_version + 1, \
                                 updated_at = datetime('now') \
             WHERE id = ? \
             RETURNING {COLS}"
        ))
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: i64) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}

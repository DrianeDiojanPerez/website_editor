use async_trait::async_trait;
use serde_json::Value;
use sqlx::types::Json;
use sqlx::SqlitePool;

use crate::packages::model::project_version::ProjectVersion;
use crate::packages::repository::{RepoError, RepoResult};

#[async_trait]
pub trait ProjectVersionRepository: Send + Sync {
    async fn list_by_project(&self, project_id: i64) -> RepoResult<Vec<ProjectVersion>>;
    async fn get(&self, id: i64) -> RepoResult<ProjectVersion>;
    async fn get_by_number(
        &self,
        project_id: i64,
        version_number: i64,
    ) -> RepoResult<ProjectVersion>;
    async fn create(
        &self,
        project_id: i64,
        version_number: i64,
        object_snapshot: &Value,
        created_by: Option<i64>,
    ) -> RepoResult<ProjectVersion>;
}

pub struct SqliteProjectVersionRepository {
    db: SqlitePool,
}

impl SqliteProjectVersionRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

const COLS: &str =
    "id, project_id, version_number, object_snapshot, created_by, created_at";

#[async_trait]
impl ProjectVersionRepository for SqliteProjectVersionRepository {
    #[tracing::instrument(skip(self))]
    async fn list_by_project(&self, project_id: i64) -> RepoResult<Vec<ProjectVersion>> {
        let rows = sqlx::query_as::<_, ProjectVersion>(&format!(
            "SELECT {COLS} FROM project_versions \
             WHERE project_id = ? ORDER BY version_number DESC"
        ))
        .bind(project_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> RepoResult<ProjectVersion> {
        sqlx::query_as::<_, ProjectVersion>(&format!(
            "SELECT {COLS} FROM project_versions WHERE id = ?"
        ))
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_number(
        &self,
        project_id: i64,
        version_number: i64,
    ) -> RepoResult<ProjectVersion> {
        sqlx::query_as::<_, ProjectVersion>(&format!(
            "SELECT {COLS} FROM project_versions \
             WHERE project_id = ? AND version_number = ?"
        ))
        .bind(project_id)
        .bind(version_number)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self, object_snapshot))]
    async fn create(
        &self,
        project_id: i64,
        version_number: i64,
        object_snapshot: &Value,
        created_by: Option<i64>,
    ) -> RepoResult<ProjectVersion> {
        let row = sqlx::query_as::<_, ProjectVersion>(&format!(
            "INSERT INTO project_versions (project_id, version_number, object_snapshot, created_by) \
             VALUES (?, ?, ?, ?) RETURNING {COLS}"
        ))
        .bind(project_id)
        .bind(version_number)
        .bind(Json(object_snapshot))
        .bind(created_by)
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }
}

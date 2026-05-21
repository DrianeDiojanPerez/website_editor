use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::packages::model::project_member::ProjectMember;
use crate::packages::repository::{RepoError, RepoResult};

#[async_trait]
pub trait ProjectMemberRepository: Send + Sync {
    async fn list_by_project(&self, project_id: i64) -> RepoResult<Vec<ProjectMember>>;
    async fn get(&self, id: i64) -> RepoResult<ProjectMember>;
    async fn attach(
        &self,
        project_id: i64,
        user_id: i64,
        role: &str,
    ) -> RepoResult<ProjectMember>;
    async fn update_role(&self, id: i64, role: &str) -> RepoResult<ProjectMember>;
    async fn detach(&self, id: i64) -> RepoResult<()>;
}

pub struct SqliteProjectMemberRepository {
    db: SqlitePool,
}

impl SqliteProjectMemberRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

const COLS: &str = "id, project_id, user_id, role";

#[async_trait]
impl ProjectMemberRepository for SqliteProjectMemberRepository {
    #[tracing::instrument(skip(self))]
    async fn list_by_project(&self, project_id: i64) -> RepoResult<Vec<ProjectMember>> {
        let rows = sqlx::query_as::<_, ProjectMember>(&format!(
            "SELECT {COLS} FROM project_members WHERE project_id = ? ORDER BY id"
        ))
        .bind(project_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> RepoResult<ProjectMember> {
        sqlx::query_as::<_, ProjectMember>(&format!(
            "SELECT {COLS} FROM project_members WHERE id = ?"
        ))
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn attach(
        &self,
        project_id: i64,
        user_id: i64,
        role: &str,
    ) -> RepoResult<ProjectMember> {
        let row = sqlx::query_as::<_, ProjectMember>(&format!(
            "INSERT INTO project_members (project_id, user_id, role) VALUES (?, ?, ?) \
             RETURNING {COLS}"
        ))
        .bind(project_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    #[tracing::instrument(skip(self))]
    async fn update_role(&self, id: i64, role: &str) -> RepoResult<ProjectMember> {
        sqlx::query_as::<_, ProjectMember>(&format!(
            "UPDATE project_members SET role = ? WHERE id = ? RETURNING {COLS}"
        ))
        .bind(role)
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn detach(&self, id: i64) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM project_members WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}

use std::sync::Arc;

use async_trait::async_trait;

use crate::packages::dto::project_version::{NewProjectVersionDto, ProjectVersionDto};
use crate::packages::repository::Store;
use crate::packages::service::ServiceResult;

#[async_trait]
pub trait ProjectVersionService: Send + Sync {
    async fn list_by_project(&self, project_id: i64) -> ServiceResult<Vec<ProjectVersionDto>>;
    async fn get(&self, id: i64) -> ServiceResult<ProjectVersionDto>;
    async fn snapshot(
        &self,
        project_id: i64,
        dto: NewProjectVersionDto,
    ) -> ServiceResult<ProjectVersionDto>;
}

pub struct ProjectVersionServiceImpl {
    store: Arc<dyn Store>,
}

impl ProjectVersionServiceImpl {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}

pub fn new_project_version_service(store: Arc<dyn Store>) -> Arc<dyn ProjectVersionService> {
    Arc::new(ProjectVersionServiceImpl::new(store))
}

#[async_trait]
impl ProjectVersionService for ProjectVersionServiceImpl {
    #[tracing::instrument(skip(self))]
    async fn list_by_project(
        &self,
        project_id: i64,
    ) -> ServiceResult<Vec<ProjectVersionDto>> {
        let rows = self
            .store
            .project_version_store()
            .list_by_project(project_id)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> ServiceResult<ProjectVersionDto> {
        Ok(self.store.project_version_store().get(id).await?.into())
    }

    // Bump the project's current_version and append a snapshot row in one shot.
    #[tracing::instrument(skip(self, dto))]
    async fn snapshot(
        &self,
        project_id: i64,
        dto: NewProjectVersionDto,
    ) -> ServiceResult<ProjectVersionDto> {
        let project = self.store.project_store().bump_version(project_id).await?;
        let row = self
            .store
            .project_version_store()
            .create(
                project_id,
                project.current_version,
                &dto.object_snapshot,
                dto.created_by,
            )
            .await?;
        Ok(row.into())
    }
}

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::packages::dto::project::{NewProjectDto, ProjectDto, UpdateProjectDto};
use crate::packages::repository::Store;
use crate::packages::service::{err_validation, ServiceResult};

#[async_trait]
pub trait ProjectService: Send + Sync {
    async fn list(&self) -> ServiceResult<Vec<ProjectDto>>;
    async fn get(&self, id: i64) -> ServiceResult<ProjectDto>;
    async fn create(&self, dto: NewProjectDto) -> ServiceResult<ProjectDto>;
    async fn update(&self, id: i64, dto: UpdateProjectDto) -> ServiceResult<ProjectDto>;
    async fn delete(&self, id: i64) -> ServiceResult<()>;
}

pub struct ProjectServiceImpl {
    store: Arc<dyn Store>,
}

impl ProjectServiceImpl {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}

pub fn new_project_service(store: Arc<dyn Store>) -> Arc<dyn ProjectService> {
    Arc::new(ProjectServiceImpl::new(store))
}

#[async_trait]
impl ProjectService for ProjectServiceImpl {
    #[tracing::instrument(skip(self))]
    async fn list(&self) -> ServiceResult<Vec<ProjectDto>> {
        let rows = self.store.project_store().list().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> ServiceResult<ProjectDto> {
        Ok(self.store.project_store().get(id).await?.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn create(&self, dto: NewProjectDto) -> ServiceResult<ProjectDto> {
        if dto.name.trim().is_empty() {
            return Err(err_validation("name must not be empty"));
        }
        let data: Value = dto.object_data.unwrap_or_else(|| json!({}));
        let row = self
            .store
            .project_store()
            .create(dto.name.trim(), &data)
            .await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn update(&self, id: i64, dto: UpdateProjectDto) -> ServiceResult<ProjectDto> {
        let repo = self.store.project_store();
        let existing = repo.get(id).await?;

        let name = dto.name.unwrap_or(existing.name);
        if name.trim().is_empty() {
            return Err(err_validation("name must not be empty"));
        }
        let data = dto.object_data.unwrap_or(existing.object_data.0);

        let row = repo.update(id, &name, &data).await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: i64) -> ServiceResult<()> {
        self.store.project_store().delete(id).await?;
        Ok(())
    }
}

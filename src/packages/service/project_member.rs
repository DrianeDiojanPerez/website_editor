use std::sync::Arc;

use async_trait::async_trait;

use crate::packages::dto::project_member::{
    AttachMemberDto, ProjectMemberDto, UpdateMemberDto,
};
use crate::packages::repository::Store;
use crate::packages::service::ServiceResult;

#[async_trait]
pub trait ProjectMemberService: Send + Sync {
    async fn list_by_project(&self, project_id: i64) -> ServiceResult<Vec<ProjectMemberDto>>;
    async fn get(&self, id: i64) -> ServiceResult<ProjectMemberDto>;
    async fn attach(
        &self,
        project_id: i64,
        dto: AttachMemberDto,
    ) -> ServiceResult<ProjectMemberDto>;
    async fn update_role(
        &self,
        id: i64,
        dto: UpdateMemberDto,
    ) -> ServiceResult<ProjectMemberDto>;
    async fn detach(&self, id: i64) -> ServiceResult<()>;
}

pub struct ProjectMemberServiceImpl {
    store: Arc<dyn Store>,
}

impl ProjectMemberServiceImpl {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}

pub fn new_project_member_service(store: Arc<dyn Store>) -> Arc<dyn ProjectMemberService> {
    Arc::new(ProjectMemberServiceImpl::new(store))
}

#[async_trait]
impl ProjectMemberService for ProjectMemberServiceImpl {
    #[tracing::instrument(skip(self))]
    async fn list_by_project(&self, project_id: i64) -> ServiceResult<Vec<ProjectMemberDto>> {
        let rows = self
            .store
            .project_member_store()
            .list_by_project(project_id)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> ServiceResult<ProjectMemberDto> {
        Ok(self.store.project_member_store().get(id).await?.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn attach(
        &self,
        project_id: i64,
        dto: AttachMemberDto,
    ) -> ServiceResult<ProjectMemberDto> {
        let row = self
            .store
            .project_member_store()
            .attach(project_id, dto.user_id, &dto.role)
            .await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn update_role(
        &self,
        id: i64,
        dto: UpdateMemberDto,
    ) -> ServiceResult<ProjectMemberDto> {
        Ok(self
            .store
            .project_member_store()
            .update_role(id, &dto.role)
            .await?
            .into())
    }

    #[tracing::instrument(skip(self))]
    async fn detach(&self, id: i64) -> ServiceResult<()> {
        self.store.project_member_store().detach(id).await?;
        Ok(())
    }
}

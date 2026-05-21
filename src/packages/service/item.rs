use std::sync::Arc;

use async_trait::async_trait;

use crate::packages::dto::item::{ItemDto, NewItemDto, UpdateItemDto};
use crate::packages::repository::Store;
use crate::packages::service::{err_item_validation, ServiceResult};

#[async_trait]
pub trait ItemService: Send + Sync {
    async fn list(&self) -> ServiceResult<Vec<ItemDto>>;
    async fn get(&self, id: i64) -> ServiceResult<ItemDto>;
    async fn create(&self, dto: NewItemDto) -> ServiceResult<ItemDto>;
    async fn update(&self, id: i64, dto: UpdateItemDto) -> ServiceResult<ItemDto>;
    async fn delete(&self, id: i64) -> ServiceResult<()>;
}

pub struct ItemServiceImpl {
    store: Arc<dyn Store>,
}

impl ItemServiceImpl {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}

pub fn new_item_service(store: Arc<dyn Store>) -> Arc<dyn ItemService> {
    Arc::new(ItemServiceImpl::new(store))
}

#[async_trait]
impl ItemService for ItemServiceImpl {
    #[tracing::instrument(skip(self))]
    async fn list(&self) -> ServiceResult<Vec<ItemDto>> {
        let items = self.store.item_store().list().await?;
        Ok(items.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> ServiceResult<ItemDto> {
        Ok(self.store.item_store().get(id).await?.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn create(&self, dto: NewItemDto) -> ServiceResult<ItemDto> {
        let name = dto.name.trim();
        if name.is_empty() {
            return Err(err_item_validation("name must not be empty"));
        }
        let item = self
            .store
            .item_store()
            .create(name, dto.description.as_deref())
            .await?;
        Ok(item.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn update(&self, id: i64, dto: UpdateItemDto) -> ServiceResult<ItemDto> {
        let repo = self.store.item_store();
        let existing = repo.get(id).await?;

        let name = dto.name.unwrap_or(existing.name);
        if name.trim().is_empty() {
            return Err(err_item_validation("name must not be empty"));
        }
        let description = dto.description.or(existing.description);

        let item = repo.update(id, &name, description.as_deref()).await?;
        Ok(item.into())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: i64) -> ServiceResult<()> {
        self.store.item_store().delete(id).await?;
        Ok(())
    }
}

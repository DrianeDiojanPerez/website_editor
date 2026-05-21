use std::sync::Arc;

use async_trait::async_trait;

use crate::packages::dto::user::{NewUserDto, UpdateUserDto, UserDto};
use crate::packages::repository::Store;
use crate::packages::service::{err_validation, ServiceResult};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn list(&self) -> ServiceResult<Vec<UserDto>>;
    async fn get(&self, id: i64) -> ServiceResult<UserDto>;
    async fn create(&self, dto: NewUserDto) -> ServiceResult<UserDto>;
    async fn update(&self, id: i64, dto: UpdateUserDto) -> ServiceResult<UserDto>;
    async fn delete(&self, id: i64) -> ServiceResult<()>;
}

pub struct UserServiceImpl {
    store: Arc<dyn Store>,
}

impl UserServiceImpl {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}

pub fn new_user_service(store: Arc<dyn Store>) -> Arc<dyn UserService> {
    Arc::new(UserServiceImpl::new(store))
}

#[async_trait]
impl UserService for UserServiceImpl {
    #[tracing::instrument(skip(self))]
    async fn list(&self) -> ServiceResult<Vec<UserDto>> {
        let rows = self.store.user_store().list().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> ServiceResult<UserDto> {
        Ok(self.store.user_store().get(id).await?.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn create(&self, dto: NewUserDto) -> ServiceResult<UserDto> {
        if dto.username.trim().is_empty() {
            return Err(err_validation("username must not be empty"));
        }
        if !dto.email.contains('@') {
            return Err(err_validation("email is invalid"));
        }
        if dto.password.len() < 8 {
            return Err(err_validation("password must be at least 8 characters"));
        }
        // TODO: hash password before persisting (e.g. argon2/bcrypt).
        let row = self
            .store
            .user_store()
            .create(dto.username.trim(), dto.email.trim(), &dto.password)
            .await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn update(&self, id: i64, dto: UpdateUserDto) -> ServiceResult<UserDto> {
        let repo = self.store.user_store();
        let existing = repo.get(id).await?;

        let username = dto.username.unwrap_or(existing.username);
        let email = dto.email.unwrap_or(existing.email);
        let password = dto.password.unwrap_or(existing.password);

        let row = repo.update(id, &username, &email, &password).await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: i64) -> ServiceResult<()> {
        self.store.user_store().delete(id).await?;
        Ok(())
    }
}

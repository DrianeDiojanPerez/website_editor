use std::sync::Arc;

use async_trait::async_trait;

use crate::packages::dto::user::{NewUserDto, UpdateUserDto, UserDto};
use crate::packages::lib::password;
use crate::packages::repository::Store;
use crate::packages::service::{err_validation, internal_error, ServiceResult};

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

async fn hash_password(pwd: String) -> ServiceResult<String> {
    password::hash(pwd).await.map_err(|e| {
        tracing::error!(error = ?e, "password hash failed");
        internal_error()
    })
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

        let hashed = hash_password(dto.password).await?;
        let row = self
            .store
            .user_store()
            .create(dto.username.trim(), dto.email.trim(), &hashed)
            .await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn update(&self, id: i64, dto: UpdateUserDto) -> ServiceResult<UserDto> {
        let repo = self.store.user_store();
        let existing = repo.get(id).await?;

        let username = dto.username.unwrap_or(existing.username);
        let email = dto.email.unwrap_or(existing.email);
        let password = match dto.password {
            Some(new) if !new.is_empty() => {
                if new.len() < 8 {
                    return Err(err_validation("password must be at least 8 characters"));
                }
                hash_password(new).await?
            }
            _ => existing.password,
        };

        let row = repo.update(id, &username, &email, &password).await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: i64) -> ServiceResult<()> {
        self.store.user_store().delete(id).await?;
        Ok(())
    }
}

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};

use crate::packages::dto::auth::{
    AuthResponseDto, LoginDto, LogoutDto, RefreshDto, TokenPairDto,
};
use crate::packages::dto::user::NewUserDto;
use crate::packages::lib::jwt::{generate_refresh_token, sha256_hex, TokenManager};
use crate::packages::lib::password;
use crate::packages::model::user::User;
use crate::packages::repository::Store;
use crate::packages::service::user::UserService;
use crate::packages::service::{
    err_invalid_credentials, err_invalid_refresh_token, internal_error, ServiceResult,
};

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn signup(&self, dto: NewUserDto) -> ServiceResult<AuthResponseDto>;
    async fn login(&self, dto: LoginDto) -> ServiceResult<AuthResponseDto>;
    async fn refresh(&self, dto: RefreshDto) -> ServiceResult<AuthResponseDto>;
    async fn logout(&self, dto: LogoutDto) -> ServiceResult<()>;
}

pub struct AuthServiceImpl {
    store: Arc<dyn Store>,
    user_service: Arc<dyn UserService>,
    token_manager: Arc<TokenManager>,
}

impl AuthServiceImpl {
    pub fn new(
        store: Arc<dyn Store>,
        user_service: Arc<dyn UserService>,
        token_manager: Arc<TokenManager>,
    ) -> Self {
        Self { store, user_service, token_manager }
    }

    async fn issue_pair_for_user(&self, user: User) -> ServiceResult<AuthResponseDto> {
        let access_token = self
            .token_manager
            .issue_access(user.id, &user.username)
            .map_err(|e| {
                tracing::error!(error = ?e, "failed to issue access token");
                internal_error()
            })?;

        let raw_refresh = generate_refresh_token();
        let hashed_refresh = sha256_hex(&raw_refresh);
        let expires_at =
            (Utc::now() + Duration::days(self.token_manager.refresh_days())).naive_utc();

        self.store
            .refresh_token_store()
            .create(user.id, &hashed_refresh, expires_at)
            .await?;

        Ok(AuthResponseDto {
            user: user.into(),
            token: TokenPairDto {
                access_token,
                refresh_token: raw_refresh,
                token_type: "Bearer".to_string(),
                expires_in: self.token_manager.access_expires_in_seconds(),
                refresh_expires_in: self.token_manager.refresh_expires_in_seconds(),
            },
        })
    }
}

pub fn new_auth_service(
    store: Arc<dyn Store>,
    user_service: Arc<dyn UserService>,
    token_manager: Arc<TokenManager>,
) -> Arc<dyn AuthService> {
    Arc::new(AuthServiceImpl::new(store, user_service, token_manager))
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    #[tracing::instrument(skip(self, dto))]
    async fn signup(&self, dto: NewUserDto) -> ServiceResult<AuthResponseDto> {
        // user_service.create validates and bcrypt-hashes the password.
        let user_dto = self.user_service.create(dto).await?;
        // Pull the freshly-created user (we need the user model, not the dto).
        let user = self.store.user_store().get(user_dto.id).await?;
        self.issue_pair_for_user(user).await
    }

    #[tracing::instrument(skip(self, dto))]
    async fn login(&self, dto: LoginDto) -> ServiceResult<AuthResponseDto> {
        let user = match self.store.user_store().get_by_email(&dto.email).await {
            Ok(u) => u,
            // Don't leak whether the email exists — same error either way.
            Err(_) => return Err(err_invalid_credentials()),
        };

        if !password::verify(dto.password, user.password.clone()).await {
            return Err(err_invalid_credentials());
        }

        self.issue_pair_for_user(user).await
    }

    #[tracing::instrument(skip(self, dto))]
    async fn refresh(&self, dto: RefreshDto) -> ServiceResult<AuthResponseDto> {
        let hashed = sha256_hex(&dto.refresh_token);
        let row = self
            .store
            .refresh_token_store()
            .find_active(&hashed)
            .await
            .map_err(|_| err_invalid_refresh_token())?;

        // Rotate: revoke the presented token before issuing the new pair.
        let _ = self.store.refresh_token_store().revoke(row.id).await;

        let user = self.store.user_store().get(row.user_id).await?;
        self.issue_pair_for_user(user).await
    }

    #[tracing::instrument(skip(self, dto))]
    async fn logout(&self, dto: LogoutDto) -> ServiceResult<()> {
        let hashed = sha256_hex(&dto.refresh_token);
        if let Ok(row) = self.store.refresh_token_store().find_active(&hashed).await {
            let _ = self.store.refresh_token_store().revoke(row.id).await;
        }
        Ok(())
    }
}

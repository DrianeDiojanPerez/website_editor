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
    // `must_change_password`:
    //   * false → self-signup; user just chose their password
    //   * true  → admin created the account; user is forced to change on first use
    async fn create(
        &self,
        dto: NewUserDto,
        must_change_password: bool,
    ) -> ServiceResult<UserDto>;
    async fn update(&self, id: i64, dto: UpdateUserDto) -> ServiceResult<UserDto>;
    async fn delete(&self, id: i64) -> ServiceResult<()>;

    // Admin sets a temporary password on someone else's account and flips
    // must_change_password = true. Also revokes their refresh tokens so they
    // can't keep extending an old session.
    async fn force_password_reset(
        &self,
        target_id: i64,
        temporary_password: String,
    ) -> ServiceResult<UserDto>;
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
    async fn create(
        &self,
        dto: NewUserDto,
        must_change_password: bool,
    ) -> ServiceResult<UserDto> {
        let hashed = hash_password(dto.password).await?;
        let row = self
            .store
            .user_store()
            .create(
                dto.username.trim(),
                dto.email.trim(),
                &hashed,
                must_change_password,
            )
            .await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self, dto))]
    async fn update(&self, id: i64, dto: UpdateUserDto) -> ServiceResult<UserDto> {
        // Field-level checks (min lengths, email shape) come from the derive.

        let repo = self.store.user_store();
        let existing = repo.get(id).await?;

        let username = dto.username.unwrap_or(existing.username);
        let email = dto.email.unwrap_or(existing.email);
        let password = match dto.password {
            Some(new) => hash_password(new).await?,
            None => existing.password,
        };

        let row = repo.update(id, &username, &email, &password).await?;
        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: i64) -> ServiceResult<()> {
        self.store.user_store().delete(id).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, temporary_password))]
    async fn force_password_reset(
        &self,
        target_id: i64,
        temporary_password: String,
    ) -> ServiceResult<UserDto> {
        // Caller passes only the password string (not a Validate DTO), so we
        // check it here. The DTO at the handler boundary already validated it
        // — this is belt-and-braces.
        if temporary_password.len() < 8 {
            return Err(err_validation("temporary password must be at least 8 characters"));
        }
        let hashed = hash_password(temporary_password).await?;

        let row = self
            .store
            .user_store()
            .set_password_and_force_reset(target_id, &hashed)
            .await?;

        // Kill any active refresh tokens — the user must re-authenticate.
        let _ = self
            .store
            .refresh_token_store()
            .revoke_all_for_user(target_id)
            .await;

        Ok(row.into())
    }
}

// -------------------------------------------------------------------------
// Service-layer unit tests.
//
// These exercise `UserServiceImpl` against a *mocked* `Store` so we can:
//   * verify the password is bcrypt-hashed before reaching the repo
//   * verify the must_change_password flag is forwarded correctly
//   * verify that not-found from the repo turns into the right service error
//
// No SQLite, no real IO. Mocks come from `#[cfg_attr(test, automock)]` on the
// `Store` and `UserRepository` traits.
// -------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::packages::model::user::User;
    use crate::packages::repository::user::MockUserRepository;
    use crate::packages::repository::{MockStore, RepoError};
    use chrono::Utc;
    use mockall::predicate::*;

    fn fake_user(username: &str, email: &str, hashed_password: &str) -> User {
        let now = Utc::now().naive_utc();
        User {
            id: 1,
            username: username.to_string(),
            email: email.to_string(),
            password: hashed_password.to_string(),
            is_system: 0,
            must_change_password: 0,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    // Wraps a configured `MockUserRepository` into a `MockStore` whose
    // `user_store()` always returns it. One helper keeps the test bodies tight.
    fn store_with_user_repo(repo: MockUserRepository) -> Arc<dyn Store> {
        let repo: Arc<dyn crate::packages::repository::user::UserRepository> = Arc::new(repo);
        let mut store = MockStore::new();
        let repo_for_closure = repo.clone();
        store.expect_user_store().returning(move || repo_for_closure.clone());
        Arc::new(store)
    }

    #[tokio::test]
    async fn create_hashes_password_before_calling_repo() {
        const RAW: &str = "longenough";

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_create()
            // The .withf(...) predicate fails the test if the repo was called
            // with anything other than:
            //   * the same username + email
            //   * a password that ISN'T the plaintext (i.e. bcrypt did its job)
            //   * must_change_password = true (admin path)
            .withf(|username, email, password, must_change| {
                *username == *"alice"
                    && *email == *"alice@example.com"
                    && *password != *RAW
                    && password.starts_with("$2") // bcrypt prefix
                    && *must_change
            })
            .times(1)
            .returning(|u, e, p, _| Ok(fake_user(u, e, p)));

        let service = UserServiceImpl::new(store_with_user_repo(user_repo));

        let dto = NewUserDto {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: RAW.to_string(),
        };

        let result = service.create(dto, /* must_change_password */ true).await;
        let user = result.expect("create should succeed");
        assert_eq!(user.username, "alice");
    }

    #[tokio::test]
    async fn get_propagates_not_found_as_service_error() {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_get()
            .with(eq(42_i64))
            .returning(|_| Err(RepoError::NotFound));

        let service = UserServiceImpl::new(store_with_user_repo(user_repo));

        let err = service.get(42).await.unwrap_err();
        // RepoError::NotFound → service::err_not_found(...) → code 2001.
        assert_eq!(err.code, crate::packages::codes::ERR_RESOURCE_NOT_FOUND);
    }

    #[tokio::test]
    async fn force_password_reset_rejects_too_short_passwords() {
        // No repo calls expected — should bail in validation before reaching it.
        let user_repo = MockUserRepository::new();
        let service = UserServiceImpl::new(store_with_user_repo(user_repo));

        let err = service
            .force_password_reset(1, "tiny".to_string())
            .await
            .unwrap_err();
        assert_eq!(err.code, crate::packages::codes::ERR_VALIDATION);
    }
}

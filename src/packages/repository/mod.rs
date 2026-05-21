pub mod project;
pub mod project_member;
pub mod project_version;
pub mod user;

use std::sync::Arc;

use sqlx::SqlitePool;
use thiserror::Error;

use self::project::{ProjectRepository, SqliteProjectRepository};
use self::project_member::{ProjectMemberRepository, SqliteProjectMemberRepository};
use self::project_version::{ProjectVersionRepository, SqliteProjectVersionRepository};
use self::user::{SqliteUserRepository, UserRepository};

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub type RepoResult<T> = Result<T, RepoError>;

// Equivalent of Go's `repository.Store` — exposes one accessor per repository.
pub trait Store: Send + Sync {
    fn user_store(&self) -> Arc<dyn UserRepository>;
    fn project_store(&self) -> Arc<dyn ProjectRepository>;
    fn project_member_store(&self) -> Arc<dyn ProjectMemberRepository>;
    fn project_version_store(&self) -> Arc<dyn ProjectVersionRepository>;
}

pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl Store for SqliteStore {
    fn user_store(&self) -> Arc<dyn UserRepository> {
        Arc::new(SqliteUserRepository::new(self.pool.clone()))
    }
    fn project_store(&self) -> Arc<dyn ProjectRepository> {
        Arc::new(SqliteProjectRepository::new(self.pool.clone()))
    }
    fn project_member_store(&self) -> Arc<dyn ProjectMemberRepository> {
        Arc::new(SqliteProjectMemberRepository::new(self.pool.clone()))
    }
    fn project_version_store(&self) -> Arc<dyn ProjectVersionRepository> {
        Arc::new(SqliteProjectVersionRepository::new(self.pool.clone()))
    }
}

pub fn new_store(pool: SqlitePool) -> Arc<dyn Store> {
    Arc::new(SqliteStore::new(pool))
}

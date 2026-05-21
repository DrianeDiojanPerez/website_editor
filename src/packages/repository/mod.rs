pub mod item;

use std::sync::Arc;

use sqlx::SqlitePool;
use thiserror::Error;

use self::item::{ItemRepository, SqliteItemRepository};

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
    fn item_store(&self) -> Arc<dyn ItemRepository>;
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
    fn item_store(&self) -> Arc<dyn ItemRepository> {
        Arc::new(SqliteItemRepository::new(self.pool.clone()))
    }
}

pub fn new_store(pool: SqlitePool) -> Arc<dyn Store> {
    Arc::new(SqliteStore::new(pool))
}

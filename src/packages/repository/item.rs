use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::packages::model::item::Item;
use crate::packages::repository::{RepoError, RepoResult};

#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn list(&self) -> RepoResult<Vec<Item>>;
    async fn get(&self, id: i64) -> RepoResult<Item>;
    async fn create(&self, name: &str, description: Option<&str>) -> RepoResult<Item>;
    async fn update(&self, id: i64, name: &str, description: Option<&str>) -> RepoResult<Item>;
    async fn delete(&self, id: i64) -> RepoResult<()>;
}

pub struct SqliteItemRepository {
    db: SqlitePool,
}

impl SqliteItemRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ItemRepository for SqliteItemRepository {
    #[tracing::instrument(skip(self))]
    async fn list(&self) -> RepoResult<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            "SELECT id, name, description, created_at, updated_at \
             FROM items ORDER BY id DESC",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(items)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: i64) -> RepoResult<Item> {
        sqlx::query_as::<_, Item>(
            "SELECT id, name, description, created_at, updated_at \
             FROM items WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn create(&self, name: &str, description: Option<&str>) -> RepoResult<Item> {
        let item = sqlx::query_as::<_, Item>(
            "INSERT INTO items (name, description) VALUES (?, ?) \
             RETURNING id, name, description, created_at, updated_at",
        )
        .bind(name)
        .bind(description)
        .fetch_one(&self.db)
        .await?;
        Ok(item)
    }

    #[tracing::instrument(skip(self))]
    async fn update(&self, id: i64, name: &str, description: Option<&str>) -> RepoResult<Item> {
        sqlx::query_as::<_, Item>(
            "UPDATE items SET name = ?, description = ?, updated_at = CURRENT_TIMESTAMP \
             WHERE id = ? \
             RETURNING id, name, description, created_at, updated_at",
        )
        .bind(name)
        .bind(description)
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(RepoError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: i64) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM items WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}

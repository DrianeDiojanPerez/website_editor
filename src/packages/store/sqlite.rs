use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

pub struct SqliteConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://data.db?mode=rwc".to_string(),
            max_connections: 5,
        }
    }
}

pub async fn new_sqlite_db(c: SqliteConfig) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(c.max_connections)
        .connect(&c.url)
        .await?;

    sqlx::migrate!("./database/migrations").run(&pool).await?;

    Ok(pool)
}

mod api;
mod packages;
mod utils;

use crate::packages::lib::{env, logger};
use crate::packages::repository::new_store;
use crate::packages::store::sqlite::{new_sqlite_db, SqliteConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = env::load_env() {
        eprintln!("failed to load .env: {e}");
    }

    // Keep the guard alive for the entire program — it flushes the file writer on drop.
    let _log_guard = logger::configure();

    let db_url = env::get_string("DATABASE_URL")?;
    let server_addr = env::get_string_or("SERVER_ADDR", "127.0.0.1:3000");

    tracing::info!(database_url = %db_url, server_addr = %server_addr, "starting");

    let pool = new_sqlite_db(SqliteConfig {
        url: db_url,
        max_connections: 5,
    })
    .await?;
    let store = new_store(pool);

    let handler = api::handler::configure_handlers(store);
    let app = api::routes::router(handler);

    let listener = tokio::net::TcpListener::bind(&server_addr).await?;
    tracing::info!("listening on {}", server_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

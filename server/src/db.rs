//! PostgreSQL pool and migrations — the only OSCC datastore.

use anyhow::Context;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Connections for a single-hospital deployment.
const MAX_CONNECTIONS: u32 = 10;

/// Reads `OSCC_DATABASE_URL`; absent or empty means "start without storage".
pub fn database_url() -> Option<String> {
    std::env::var("OSCC_DATABASE_URL")
        .ok()
        .filter(|url| !url.trim().is_empty())
}

/// Opens the pool.
///
/// # Errors
///
/// Returns a context-rich error when the database is unreachable.
pub async fn connect(url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect(url)
        .await
        .context("cannot connect to the OSCC database")
}

/// Applies every embedded migration.
///
/// # Errors
///
/// Returns a context-rich error when a migration fails.
pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("OSCC migration failed")
}

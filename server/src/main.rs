//! OSCC API entry point.
//!
//! Binds to loopback by default; binding to the LAN is an explicit
//! deployment decision (`OSCC_BIND`, AGENTS.md §2 rule 7). Storage is
//! optional at startup: without `OSCC_DATABASE_URL` the server runs and
//! reports the database as absent.

use std::net::SocketAddr;

use anyhow::Context;
use oscc_server::{AppState, app, db};
use tracing_subscriber::EnvFilter;

/// Loopback default: a dev machine without an explicit bind exposed nothing.
const DEFAULT_BIND: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let state = match db::database_url() {
        Some(url) => {
            let pool = db::connect(&url)
                .await
                .context("cannot connect to the OSCC database")?;
            db::migrate(&pool)
                .await
                .context("cannot prepare the OSCC database")?;
            tracing::info!("oscc database ready");
            AppState::with_pool(pool)
        }
        None => {
            tracing::warn!("OSCC_DATABASE_URL is not set: auth endpoints will answer 503");
            AppState::new()
        }
    };

    let bind = std::env::var("OSCC_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string());
    let addr: SocketAddr = bind
        .parse()
        .with_context(|| format!("invalid OSCC_BIND value: {bind}"))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("cannot bind {addr}"))?;

    tracing::info!(%addr, "oscc-server listening");
    axum::serve(listener, app(state))
        .await
        .context("oscc-server stopped with an error")?;
    Ok(())
}

/// Installs PII-free structured logging. The default filter is `info`;
/// `RUST_LOG` overrides it during development.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

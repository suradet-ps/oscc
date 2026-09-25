//! OSCC central API (AGENTS.md §3).
//!
//! The only writer of OSCC data, and the only component allowed to read
//! HOSxP. M1 builds the trust layer: append-only audit, local auth, and
//! RBAC enforcement on top of the M0 shell. The server starts even when
//! the database is absent; auth endpoints then answer `503` instead of
//! pretending.

pub mod audit;
pub mod auth;
pub mod db;
pub mod error;

use std::time::Instant;

use axum::http::{HeaderValue, Method, header};
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

pub use error::AppError;

/// Shared state for every route.
#[derive(Clone)]
pub struct AppState {
    started_at: Instant,
    pool: Option<PgPool>,
}

impl AppState {
    /// A state without storage: health works, auth answers `503`.
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            pool: None,
        }
    }

    /// A state backed by the OSCC database.
    pub fn with_pool(pool: PgPool) -> Self {
        Self {
            started_at: Instant::now(),
            pool: Some(pool),
        }
    }

    /// The database pool.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Unavailable`] when the server started without a
    /// database.
    pub fn pool(&self) -> Result<&PgPool, AppError> {
        self.pool
            .as_ref()
            .ok_or(AppError::Unavailable("database not configured"))
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds the API router.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .nest("/api/v1/auth", auth::routes::routes())
        .layer(cors_layer())
        .with_state(state)
}

/// Desktop webviews only: the Tauri origin plus the trunk dev server.
/// Bearer tokens travel in headers, so credentials are never allowed.
fn cors_layer() -> CorsLayer {
    let origins = [
        "http://tauri.localhost",
        "tauri://localhost",
        "http://localhost:1420",
        "http://127.0.0.1:1420",
    ]
    .into_iter()
    .filter_map(|origin| origin.parse::<HeaderValue>().ok())
    .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

/// Everything a probe needs — no PII, no internals.
#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
    uptime_secs: u64,
    database: bool,
}

/// Liveness probe: process up, version, uptime, database presence.
async fn healthz(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        uptime_secs: state.started_at.elapsed().as_secs(),
        database: state.pool.is_some(),
    })
}

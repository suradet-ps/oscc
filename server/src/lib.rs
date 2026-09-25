//! OSCC central API (AGENTS.md §3).
//!
//! The only writer of OSCC data, and the only component allowed to read
//! HOSxP. M0 ships the app shell and the health endpoint; authentication,
//! RBAC, the append-only audit, and the database arrive in M1.

use std::time::Instant;

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

/// Shared state for every route.
#[derive(Clone)]
pub struct AppState {
    started_at: Instant,
}

impl AppState {
    /// A fresh state with the uptime clock started.
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Everything a probe needs — no PII, no internals.
#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
    uptime_secs: u64,
}

/// Builds the API router.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(state)
}

/// Liveness probe: process up, version, uptime. Safe to expose on the LAN.
async fn healthz(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        uptime_secs: state.started_at.elapsed().as_secs(),
    })
}

//! MySQL connection pool with an enforced read-only session.
//!
//! TLS is opportunistic by default: the pilot HOSxP instance has TLS
//! disabled (verified against the live server), so a hard `Required` would
//! simply make the connection impossible. This is a documented residual —
//! the read-only grant, the session mode, and the SQL guard remain the
//! enforced boundaries — and `OSCC_HOSXP_SSL_MODE` raises the bar without a
//! code change once the DBA enables TLS.

use std::time::Duration;

use secrecy::ExposeSecret;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlSslMode};

use crate::config::HosxConfig;
use crate::error::Error;
use crate::readonly_guard::{PING_SQL, READ_ONLY_SESSION_SQL};

/// Pool size: a single hospital server serving a handful of workstations.
const MAX_CONNECTIONS: u32 = 5;

/// How long to wait for a connection before reporting the database down.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Default TLS posture: use TLS when the HOSxP server offers it, fall back
/// to an unencrypted channel when it does not. The credentials stay on the
/// hospital LAN and are protected by the read-only account itself.
const DEFAULT_SSL_MODE: MySqlSslMode = MySqlSslMode::Preferred;

/// Server-side SELECT timeout: a pathological query cannot hang a shift.
/// Best-effort — servers without the variable ignore the error.
const STATEMENT_TIMEOUT_SESSION_SQL: &str = "SET SESSION max_execution_time = 5000";

/// Opens a pool of read-only MySQL connections.
///
/// Every new connection immediately runs `SET SESSION TRANSACTION READ
/// ONLY`, so the session itself rejects DML even if a non-SELECT statement
/// slips through the application-level guard (AGENTS.md §2 rule 1).
///
/// # Errors
///
/// Returns [`Error::Connect`] when the pool cannot be opened.
pub async fn connect(cfg: &HosxConfig) -> Result<MySqlPool, Error> {
    let options = MySqlConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .database(&cfg.database)
        .username(&cfg.user)
        .ssl_mode(configured_ssl_mode())
        // sqlx keeps its own plaintext copy inside the pool for reconnects;
        // documented residual, everything under our control is zeroized.
        .password(cfg.password.expose_secret());

    MySqlPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query(READ_ONLY_SESSION_SQL)
                    .execute(&mut *conn)
                    .await?;
                let _ = sqlx::query(STATEMENT_TIMEOUT_SESSION_SQL)
                    .execute(&mut *conn)
                    .await;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .map_err(Error::Connect)
}

/// Verifies that the pool can reach HOSxP and that the session is read-only.
///
/// # Errors
///
/// Returns [`Error::Database`] when the ping fails.
pub async fn ping(pool: &MySqlPool) -> Result<(), Error> {
    sqlx::query(PING_SQL)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(Error::Database)
}

/// The TLS posture in effect, read from `OSCC_HOSXP_SSL_MODE`.
///
/// Accepted values (case-insensitive): `preferred` (default), `required`,
/// `verify_ca`, `verify_identity`. Anything else falls back to `preferred`.
fn configured_ssl_mode() -> MySqlSslMode {
    ssl_mode_from(std::env::var("OSCC_HOSXP_SSL_MODE").ok().as_deref())
}

/// Pure resolver for the TLS posture, so the mapping is testable.
fn ssl_mode_from(raw: Option<&str>) -> MySqlSslMode {
    match raw.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("required") => MySqlSslMode::Required,
        Some("verify_ca") => MySqlSslMode::VerifyCa,
        Some("verify_identity") => MySqlSslMode::VerifyIdentity,
        _ => DEFAULT_SSL_MODE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssl_mode_defaults_to_preferred() {
        assert!(matches!(ssl_mode_from(None), MySqlSslMode::Preferred));
        assert!(matches!(
            ssl_mode_from(Some("nonsense")),
            MySqlSslMode::Preferred
        ));
        assert!(matches!(ssl_mode_from(Some("  ")), MySqlSslMode::Preferred));
    }

    #[test]
    fn ssl_mode_accepts_explicit_modes_case_insensitively() {
        assert!(matches!(
            ssl_mode_from(Some("required")),
            MySqlSslMode::Required
        ));
        assert!(matches!(
            ssl_mode_from(Some(" VERIFY_CA ")),
            MySqlSslMode::VerifyCa
        ));
        assert!(matches!(
            ssl_mode_from(Some("verify_identity")),
            MySqlSslMode::VerifyIdentity
        ));
    }
}

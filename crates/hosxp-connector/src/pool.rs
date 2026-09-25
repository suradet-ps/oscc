//! MySQL connection pool with an enforced read-only session and TLS.

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

/// Minimum TLS posture: the channel must be encrypted. `Required` never
/// falls back to plaintext; certificate verification (`VerifyCa`) is the
/// follow-up once the hospital CA is available (docs/runbook.md).
const SSL_MODE: MySqlSslMode = MySqlSslMode::Required;

/// Server-side SELECT timeout: a pathological query cannot hang a shift.
/// Best-effort — servers without the variable ignore the error.
const STATEMENT_TIMEOUT_SESSION_SQL: &str = "SET SESSION max_execution_time = 5000";

/// Opens a pool of read-only MySQL connections.
///
/// The channel is encrypted by contract: a server without TLS makes the
/// connection fail instead of silently sending credentials in plaintext.
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
        .ssl_mode(SSL_MODE)
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

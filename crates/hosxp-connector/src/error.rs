//! Connector error type.

use thiserror::Error;

use crate::config::ConfigError;

/// A connector failure. Messages stay free of credentials and parameter
/// values (AGENTS.md §2 rule 2).
#[derive(Debug, Error)]
pub enum Error {
    /// The configuration is missing or invalid.
    #[error("connector configuration error: {0}")]
    Config(#[from] ConfigError),
    /// The lookup input is malformed; nothing was sent to the database.
    #[error("invalid patient query: {0}")]
    InvalidQuery(&'static str),
    /// The read-only guard rejected a statement (a bug, not user error).
    #[error(transparent)]
    Guard(#[from] crate::readonly_guard::GuardError),
    /// The pool could not open the connection (network, credentials, TLS).
    #[error("could not connect to HOSxP")]
    Connect(#[source] sqlx::Error),
    /// A statement failed on an open connection.
    #[error("HOSxP statement failed")]
    Database(#[source] sqlx::Error),
}

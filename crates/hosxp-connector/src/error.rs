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
    /// The pool could not open the connection (network, credentials, TLS).
    #[error("could not connect to HOSxP")]
    Connect(#[source] sqlx::Error),
    /// A statement failed on an open connection.
    #[error("HOSxP statement failed")]
    Database(#[source] sqlx::Error),
}

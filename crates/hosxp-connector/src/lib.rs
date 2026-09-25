//! The only crate allowed to talk to HOSxP (AGENTS.md §2 rule 1).
//!
//! HOSxP is read-only, always: a dedicated `GRANT SELECT` user, a read-only
//! session, the SQL guard in [`readonly_guard`], and parameterized queries
//! on top. The pool requires TLS (`ssl-mode=REQUIRED`), and the password is
//! a `SecretString` end to end.

pub mod config;
pub mod error;
pub mod pool;
pub mod readonly_guard;

pub use config::HosxConfig;
pub use error::Error;
pub use readonly_guard::{GuardError, PING_SQL, READ_ONLY_SESSION_SQL, assert_read_only};

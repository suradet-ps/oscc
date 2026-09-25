//! The only crate allowed to talk to HOSxP (AGENTS.md §2 rule 1).
//!
//! HOSxP is read-only, always: a dedicated `GRANT SELECT` user, a read-only
//! session, the SQL guard in [`readonly_guard`], and parameterized queries
//! on top. M0 ships the guard and the session constants; the configuration
//! and connection pool arrive in M1.

pub mod readonly_guard;

pub use readonly_guard::{GuardError, PING_SQL, READ_ONLY_SESSION_SQL, assert_read_only};

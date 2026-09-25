//! OSCC pure logic.
//!
//! Everything testable without a database, a server, or a clock: case
//! lifecycle transitions, deadline windows and urgency, and role-based
//! access decisions. Handlers and storage call into these functions; they
//! never reimplement the rules (AGENTS.md §5).

mod rbac;
mod session;
mod timers;
mod transition;

pub use rbac::{Permission, allows, permissions, requires_reason};
pub use session::{
    DEFAULT_SESSION_POLICY, SessionPolicy, SessionState, WARNING_LEAD_SECONDS,
    seconds_until_expiry, session_state,
};
pub use timers::{Urgency, due_at, urgency_at, warn_lead, window};
pub use transition::{TransitionError, can_transition, transition};

//! Session lifetime rules (AGENTS.md §6, DESIGN.md session warning).
//!
//! Absolute and idle limits are pure data: the server stores timestamps and
//! asks these functions for a verdict, so the policy is testable without a
//! clock and without a database.

use chrono::{DateTime, Duration, Utc};

/// When sessions expire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionPolicy {
    /// Hard limit from login, regardless of activity.
    pub absolute: Duration,
    /// Limit measured from the last seen request.
    pub idle: Duration,
}

/// The Phase 1 policy: one shift, with a short idle lock.
pub const DEFAULT_SESSION_POLICY: SessionPolicy = SessionPolicy {
    absolute: Duration::hours(8),
    idle: Duration::minutes(30),
};

/// How long before expiry the client warns and offers to extend.
pub const WARNING_LEAD_SECONDS: i64 = 60;

/// How a stored session reads at a given instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Usable.
    Active,
    /// The absolute limit passed.
    ExpiredAbsolute,
    /// The idle limit passed.
    ExpiredIdle,
    /// Explicitly revoked (logout).
    Revoked,
}

/// Evaluates a session at `now`.
///
/// Revocation wins over expiry; absolute expiry wins over idle expiry so
/// the audit trail can say why the session ended.
pub fn session_state(
    now: DateTime<Utc>,
    created_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    policy: SessionPolicy,
) -> SessionState {
    if revoked_at.is_some() {
        return SessionState::Revoked;
    }
    if now >= created_at + policy.absolute {
        return SessionState::ExpiredAbsolute;
    }
    if now >= last_seen_at + policy.idle {
        return SessionState::ExpiredIdle;
    }
    SessionState::Active
}

/// Seconds until the session locks — the nearer of the two limits, clamped
/// at zero for the client countdown (DESIGN.md session warning).
pub fn seconds_until_expiry(
    now: DateTime<Utc>,
    created_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    policy: SessionPolicy,
) -> i64 {
    let absolute_remaining = (created_at + policy.absolute - now).num_seconds();
    let idle_remaining = (last_seen_at + policy.idle - now).num_seconds();
    absolute_remaining.min(idle_remaining).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 25, hour, minute, 0)
            .single()
            .expect("valid test timestamp")
    }

    #[test]
    fn fresh_session_is_active() {
        let state = session_state(at(8, 10), at(8, 0), at(8, 10), None, DEFAULT_SESSION_POLICY);
        assert_eq!(state, SessionState::Active);
    }

    #[test]
    fn idle_limit_expires_the_session() {
        let state = session_state(at(9, 0), at(8, 0), at(8, 29), None, DEFAULT_SESSION_POLICY);
        assert_eq!(state, SessionState::ExpiredIdle);
    }

    #[test]
    fn absolute_limit_wins_even_when_recently_seen() {
        let state = session_state(
            at(16, 1),
            at(8, 0),
            at(15, 59),
            None,
            DEFAULT_SESSION_POLICY,
        );
        assert_eq!(state, SessionState::ExpiredAbsolute);
    }

    #[test]
    fn revoked_wins_over_expiry() {
        let state = session_state(
            at(16, 1),
            at(8, 0),
            at(8, 1),
            Some(at(8, 2)),
            DEFAULT_SESSION_POLICY,
        );
        assert_eq!(state, SessionState::Revoked);
    }

    #[test]
    fn expiry_boundary_is_inclusive() {
        let state = session_state(at(8, 30), at(8, 0), at(8, 0), None, DEFAULT_SESSION_POLICY);
        assert_eq!(state, SessionState::ExpiredIdle);
    }

    #[test]
    fn countdown_reports_the_nearer_limit() {
        assert_eq!(
            seconds_until_expiry(at(8, 10), at(8, 0), at(8, 10), DEFAULT_SESSION_POLICY),
            30 * 60
        );
        assert_eq!(
            seconds_until_expiry(at(15, 59), at(8, 0), at(15, 30), DEFAULT_SESSION_POLICY),
            60
        );
    }

    #[test]
    fn countdown_never_goes_negative() {
        assert_eq!(
            seconds_until_expiry(at(17, 0), at(8, 0), at(8, 1), DEFAULT_SESSION_POLICY),
            0
        );
    }
}

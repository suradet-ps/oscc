//! Case lifecycle transitions (AGENTS.md §5).
//!
//! Phase 1 lifecycle: `Intake -> Active -> FollowUp -> Active|Closed`.
//! `Closed` is terminal; everything else must go through a transition
//! function so the server can write the matching audit entry.

use oscc_models::CaseStatus;
use thiserror::Error;

/// A rejected lifecycle transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("cannot move a case from {from:?} to {to:?}")]
pub struct TransitionError {
    /// The state the case is in.
    pub from: CaseStatus,
    /// The requested next state.
    pub to: CaseStatus,
}

/// Returns `true` when `to` is a legal next state for `from`.
pub fn can_transition(from: CaseStatus, to: CaseStatus) -> bool {
    matches!(
        (from, to),
        (CaseStatus::Intake, CaseStatus::Active | CaseStatus::Closed)
            | (
                CaseStatus::Active,
                CaseStatus::FollowUp | CaseStatus::Closed
            )
            | (
                CaseStatus::FollowUp,
                CaseStatus::Active | CaseStatus::Closed
            )
    )
}

/// Applies a lifecycle transition.
///
/// # Errors
///
/// Returns [`TransitionError`] when the move is not allowed.
pub fn transition(from: CaseStatus, to: CaseStatus) -> Result<CaseStatus, TransitionError> {
    if can_transition(from, to) {
        Ok(to)
    } else {
        Err(TransitionError { from, to })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_transitions_pass() {
        for (from, to) in [
            (CaseStatus::Intake, CaseStatus::Active),
            (CaseStatus::Intake, CaseStatus::Closed),
            (CaseStatus::Active, CaseStatus::FollowUp),
            (CaseStatus::Active, CaseStatus::Closed),
            (CaseStatus::FollowUp, CaseStatus::Active),
            (CaseStatus::FollowUp, CaseStatus::Closed),
        ] {
            assert!(
                transition(from, to).is_ok(),
                "expected {from:?} -> {to:?} to be allowed"
            );
        }
    }

    #[test]
    fn closed_is_terminal() {
        for to in [
            CaseStatus::Intake,
            CaseStatus::Active,
            CaseStatus::FollowUp,
            CaseStatus::Closed,
        ] {
            assert!(
                transition(CaseStatus::Closed, to).is_err(),
                "expected Closed -> {to:?} to be rejected"
            );
        }
    }

    #[test]
    fn backwards_and_self_transitions_are_rejected() {
        assert!(transition(CaseStatus::Active, CaseStatus::Intake).is_err());
        assert!(transition(CaseStatus::Active, CaseStatus::Active).is_err());
        assert!(transition(CaseStatus::Intake, CaseStatus::FollowUp).is_err());
    }

    #[test]
    fn error_reports_both_states() {
        let err = transition(CaseStatus::Intake, CaseStatus::FollowUp).expect_err("rejected");
        assert_eq!(err.from, CaseStatus::Intake);
        assert_eq!(err.to, CaseStatus::FollowUp);
    }
}

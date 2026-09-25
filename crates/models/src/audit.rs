//! Auditable actions (AGENTS.md §2 rule 4).
//!
//! Every action in this list must produce an audit entry; a case action
//! without an audit path is a bug, not a feature request.

use serde::{Deserialize, Serialize};

/// What a user did to a case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// A new case was registered.
    CaseOpened,
    /// A case was opened in the workspace.
    CaseViewed,
    /// Identity was unmasked through the reveal dialog.
    IdentityRevealed,
    /// Identity was masked again.
    IdentityRemasked,
    /// Case fields were edited.
    CaseUpdated,
    /// A deadline task was recorded as done.
    TaskCompleted,
    /// A deadline task was waived, with a reason.
    TaskWaived,
    /// Emergency access was taken, with a reason.
    BreakGlassOpened,
    /// The case was closed.
    CaseClosed,
    /// Aggregate data was exported.
    AggregateExported,
}

impl AuditAction {
    /// The stable machine name used by storage and the API.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CaseOpened => "case_opened",
            Self::CaseViewed => "case_viewed",
            Self::IdentityRevealed => "identity_revealed",
            Self::IdentityRemasked => "identity_remasked",
            Self::CaseUpdated => "case_updated",
            Self::TaskCompleted => "task_completed",
            Self::TaskWaived => "task_waived",
            Self::BreakGlassOpened => "break_glass_opened",
            Self::CaseClosed => "case_closed",
            Self::AggregateExported => "aggregate_exported",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_actions_serialize_snake_case() {
        let json = serde_json::to_string(&AuditAction::IdentityRevealed).expect("serialize");
        assert_eq!(json, "\"identity_revealed\"");
        assert_eq!(AuditAction::BreakGlassOpened.as_str(), "break_glass_opened");
    }
}

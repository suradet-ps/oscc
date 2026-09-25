//! Deadline tasks that drive the deadline rail (DESIGN.md signature element).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The kinds of time-critical task a case can carry.
///
/// Windows and warning leads live in `oscc-core::timers`; the exact policy
/// still needs OSCC-lead sign-off (AGENTS.md §12 item 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// HIV post-exposure prophylaxis, effective within 72 hours.
    Pep72h,
    /// Emergency contraception, effective within 120 hours.
    Ec120h,
    /// STI prophylaxis.
    StiProphylaxis,
    /// Hepatitis B prophylaxis.
    Hbv,
    /// First follow-up appointment (one week).
    FollowUp1w,
    /// Second follow-up appointment (four weeks).
    FollowUp4w,
}

impl TaskKind {
    /// The stable machine name used by storage and the API.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pep72h => "pep_72h",
            Self::Ec120h => "ec_120h",
            Self::StiProphylaxis => "sti_prophylaxis",
            Self::Hbv => "hbv",
            Self::FollowUp1w => "follow_up_1w",
            Self::FollowUp4w => "follow_up_4w",
        }
    }
}

/// Lifecycle of a single deadline task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    /// Not yet done; urgency is derived from the clock.
    Pending,
    /// Recorded as given or scheduled.
    Done,
    /// The window passed without a recorded outcome.
    Missed,
    /// Deliberately skipped, with a recorded reason (audited).
    Waived,
}

/// One deadline on the rail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseTask {
    /// Which clinical deadline this is.
    pub kind: TaskKind,
    /// Absolute due time, computed from the intake anchor by `oscc-core`.
    pub due_at: DateTime<Utc>,
    /// Current state.
    pub state: TaskState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_kind_machine_names_are_stable() {
        assert_eq!(TaskKind::Pep72h.as_str(), "pep_72h");
        assert_eq!(TaskKind::FollowUp4w.as_str(), "follow_up_4w");
    }

    #[test]
    fn task_serde_is_snake_case() {
        let json = serde_json::to_string(&TaskKind::StiProphylaxis).expect("serialize");
        assert_eq!(json, "\"sti_prophylaxis\"");
        let state = serde_json::to_string(&TaskState::Missed).expect("serialize");
        assert_eq!(state, "\"missed\"");
    }
}

//! Deadline windows and urgency for the deadline rail.
//!
//! Windows marked *provisional* are clinically reasonable defaults; the
//! OSCC lead must sign them off before the pilot (AGENTS.md §12 item 5).
//! The UI only renders what these functions return, so a policy change is
//! a change here plus tests - never a change in a handler.

use chrono::{DateTime, Duration, Utc};
use oscc_models::{CaseTask, TaskKind, TaskState};

/// How long after the intake anchor the intervention stops being effective.
///
/// `StiProphylaxis` and `Hbv` windows are provisional pending sign-off.
pub fn window(kind: TaskKind) -> Duration {
    match kind {
        TaskKind::Pep72h => Duration::hours(72),
        TaskKind::Ec120h => Duration::hours(120),
        TaskKind::StiProphylaxis => Duration::hours(72),
        TaskKind::Hbv => Duration::hours(24),
        TaskKind::FollowUp1w => Duration::days(7),
        TaskKind::FollowUp4w => Duration::days(28),
    }
}

/// How early before the deadline the chip turns amber (DESIGN.md urgency).
pub fn warn_lead(kind: TaskKind) -> Duration {
    match kind {
        TaskKind::Pep72h => Duration::hours(24),
        TaskKind::Ec120h => Duration::hours(24),
        TaskKind::StiProphylaxis => Duration::hours(24),
        TaskKind::Hbv => Duration::hours(6),
        TaskKind::FollowUp1w => Duration::hours(48),
        TaskKind::FollowUp4w => Duration::hours(48),
    }
}

/// The absolute due time of `kind` for a case anchored at `anchor`.
pub fn due_at(anchor: DateTime<Utc>, kind: TaskKind) -> DateTime<Utc> {
    anchor + window(kind)
}

/// How a pending deadline reads on the rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    /// Comfortably ahead of the warning window.
    Normal,
    /// Inside the warning window but not yet due.
    DueSoon,
    /// The deadline has passed.
    Overdue,
}

/// The derived urgency for the rail.
///
/// Returns `None` once a task is finished or waived - the rail shows no
/// colour for settled work.
pub fn urgency_at(task: &CaseTask, now: DateTime<Utc>) -> Option<Urgency> {
    match task.state {
        TaskState::Done | TaskState::Waived => None,
        TaskState::Missed => Some(Urgency::Overdue),
        TaskState::Pending => {
            let warn_at = task.due_at - warn_lead(task.kind);
            if now >= task.due_at {
                Some(Urgency::Overdue)
            } else if now >= warn_at {
                Some(Urgency::DueSoon)
            } else {
                Some(Urgency::Normal)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn anchor() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 25, 8, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    fn task(kind: TaskKind, state: TaskState) -> CaseTask {
        CaseTask {
            kind,
            due_at: due_at(anchor(), kind),
            state,
        }
    }

    #[test]
    fn windows_match_the_documented_policy() {
        assert_eq!(window(TaskKind::Pep72h), Duration::hours(72));
        assert_eq!(window(TaskKind::Ec120h), Duration::hours(120));
        assert_eq!(window(TaskKind::Hbv), Duration::hours(24));
        assert_eq!(window(TaskKind::FollowUp4w), Duration::days(28));
    }

    #[test]
    fn due_at_offsets_the_anchor() {
        assert_eq!(
            due_at(anchor(), TaskKind::Pep72h),
            anchor() + Duration::hours(72)
        );
    }

    #[test]
    fn pending_before_warn_window_is_normal() {
        let task = task(TaskKind::Pep72h, TaskState::Pending);
        assert_eq!(
            urgency_at(&task, anchor() + Duration::hours(12)),
            Some(Urgency::Normal)
        );
    }

    #[test]
    fn pending_inside_warn_window_is_due_soon() {
        let task = task(TaskKind::Pep72h, TaskState::Pending);
        assert_eq!(
            urgency_at(&task, anchor() + Duration::hours(60)),
            Some(Urgency::DueSoon)
        );
    }

    #[test]
    fn pending_at_or_after_due_is_overdue() {
        let task = task(TaskKind::Hbv, TaskState::Pending);
        assert_eq!(
            urgency_at(&task, anchor() + Duration::hours(24)),
            Some(Urgency::Overdue)
        );
        assert_eq!(
            urgency_at(&task, anchor() + Duration::hours(30)),
            Some(Urgency::Overdue)
        );
    }

    #[test]
    fn settled_tasks_have_no_urgency() {
        for state in [TaskState::Done, TaskState::Waived] {
            let task = task(TaskKind::Pep72h, state);
            assert_eq!(urgency_at(&task, anchor()), None);
        }
    }

    #[test]
    fn missed_task_is_overdue_regardless_of_clock() {
        let task = task(TaskKind::Ec120h, TaskState::Missed);
        assert_eq!(urgency_at(&task, anchor()), Some(Urgency::Overdue));
    }
}

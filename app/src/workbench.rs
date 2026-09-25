//! Client-side case workbench state and helpers.

use chrono::{DateTime, Local, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{ApiClient, CaseDetail, CaseSummary, RevealedPatient};

/// Which queue the nav rail is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueFilter {
    /// Newly registered cases.
    Intake,
    /// The signed-in user's cases (assignment lands in a later milestone,
    /// so this currently matches everything).
    Mine,
    /// Cases awaiting follow-up.
    FollowUp,
    /// Every case.
    All,
    /// Aggregate dashboard (later milestone).
    Dashboard,
}

impl QueueFilter {
    /// Every entry, in nav order.
    pub const ALL: [QueueFilter; 5] = [
        QueueFilter::Intake,
        QueueFilter::Mine,
        QueueFilter::FollowUp,
        QueueFilter::All,
        QueueFilter::Dashboard,
    ];

    /// The Thai label for the nav rail.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Intake => "รับแจ้งใหม่",
            Self::Mine => "เคสของฉัน",
            Self::FollowUp => "ติดตามวันนี้",
            Self::All => "ทั้งหมด",
            Self::Dashboard => "แดชบอร์ด",
        }
    }

    /// Whether the entry is usable in this milestone.
    pub fn enabled(&self) -> bool {
        !matches!(self, Self::Dashboard)
    }
}

/// Shared workbench state for the queue, detail, and modals.
#[derive(Clone, Copy)]
pub struct WorkbenchState {
    /// The active queue filter.
    pub queue: RwSignal<QueueFilter>,
    /// Every loaded case (the filter is applied client-side).
    pub cases: RwSignal<Vec<CaseSummary>>,
    /// The open case, if any.
    pub selected: RwSignal<Option<CaseDetail>>,
    /// Full identity while revealed; cleared on close, filter change, and
    /// sign-out.
    pub revealed: RwSignal<Option<RevealedPatient>>,
    /// True while a request is in flight.
    pub loading: RwSignal<bool>,
    /// The last user-facing error.
    pub error: RwSignal<Option<String>>,
}

impl WorkbenchState {
    /// A fresh, empty workbench.
    pub fn new() -> Self {
        Self {
            queue: RwSignal::new(QueueFilter::All),
            cases: RwSignal::new(Vec::new()),
            selected: RwSignal::new(None),
            revealed: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(None),
        }
    }

    /// Reloads every case for the queue counts and the active filter.
    pub fn reload(self, api: ApiClient) {
        self.loading.set(true);
        self.error.set(None);
        spawn_local(async move {
            match api.list_cases(None).await {
                Ok(cases) => self.cases.set(cases),
                Err(err) => self.error.set(Some(err.thai_message().to_string())),
            }
            self.loading.set(false);
        });
    }

    /// Opens one case, masking identity again.
    pub fn open_case(self, api: ApiClient, case_id: String) {
        self.loading.set(true);
        self.error.set(None);
        self.revealed.set(None);
        spawn_local(async move {
            match api.get_case(&case_id).await {
                Ok(detail) => self.selected.set(Some(detail)),
                Err(err) => self.error.set(Some(err.thai_message().to_string())),
            }
            self.loading.set(false);
        });
    }

    /// Switches the queue filter and closes any open case.
    pub fn select_queue(self, filter: QueueFilter) {
        self.queue.set(filter);
        self.selected.set(None);
        self.revealed.set(None);
    }

    /// Closes the open case.
    pub fn close_case(self) {
        self.selected.set(None);
        self.revealed.set(None);
    }
}

impl Default for WorkbenchState {
    fn default() -> Self {
        Self::new()
    }
}

/// The cases visible under `filter`.
pub fn visible_cases(cases: &[CaseSummary], filter: QueueFilter) -> Vec<CaseSummary> {
    cases
        .iter()
        .filter(|case| match filter {
            QueueFilter::Intake => case.status == "intake",
            QueueFilter::FollowUp => case.status == "follow_up",
            QueueFilter::Mine | QueueFilter::All => true,
            QueueFilter::Dashboard => false,
        })
        .cloned()
        .collect()
}

/// The nav-rail count for `filter`.
pub fn count_for(cases: &[CaseSummary], filter: QueueFilter) -> usize {
    visible_cases(cases, filter).len()
}

/// `dd/mm/yyyy HH:MM` in the workstation's local time.
pub fn format_local(timestamp: DateTime<Utc>) -> String {
    timestamp
        .with_timezone(&Local)
        .format("%d/%m/%Y %H:%M")
        .to_string()
}

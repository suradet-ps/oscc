//! OSCC domain types.
//!
//! Inert data only: no I/O, no storage, no framework types. Behaviour
//! (lifecycle transitions, deadline rules, access decisions) belongs in
//! `oscc-core`. Every type that carries PII redacts its [`Debug`] output,
//! because AGENTS.md §2 rule 2 forbids patient data in logs.

mod audit;
mod case;
mod cid;
mod org;
mod patient;
mod task;

pub use audit::AuditAction;
pub use case::{Case, CaseId, CaseIdError, CaseStatus, IncidentType, RiskLevel};
pub use cid::{Cid, CidError};
pub use org::{Department, Role};
pub use patient::{Hn, HnError, PatientLink};
pub use task::{CaseTask, TaskKind, TaskState};

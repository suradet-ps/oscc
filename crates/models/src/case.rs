//! Case identity, lifecycle status, and intake taxonomy.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::org::Department;

/// The non-identifying case identifier, e.g. `OSCC-2026-0042`.
///
/// It is deliberately the primary label in every list view (DESIGN.md), so
/// it must stay free of patient data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CaseId(String);

/// The literal prefix every case id carries.
const PREFIX: &str = "OSCC";

/// Why a string is not a valid case id.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CaseIdError {
    /// The value is not `OSCC-<4-digit year>-<4-digit sequence>`.
    #[error("case id must look like OSCC-YYYY-NNNN")]
    Shape,
    /// The year segment is not four digits.
    #[error("case id year segment must be four digits")]
    Year,
    /// The sequence segment is not four digits.
    #[error("case id sequence segment must be four digits")]
    Sequence,
}

impl CaseId {
    /// Validates `raw` as `OSCC-<year>-<sequence>`.
    ///
    /// # Errors
    ///
    /// Returns [`CaseIdError`] describing the first segment that is wrong.
    pub fn parse(raw: &str) -> Result<Self, CaseIdError> {
        let mut parts = raw.split('-');
        let (Some(prefix), Some(year), Some(sequence), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(CaseIdError::Shape);
        };
        if prefix != PREFIX {
            return Err(CaseIdError::Shape);
        }
        if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
            return Err(CaseIdError::Year);
        }
        if sequence.len() != 4 || !sequence.chars().all(|c| c.is_ascii_digit()) {
            return Err(CaseIdError::Sequence);
        }
        Ok(Self(raw.to_string()))
    }

    /// The id as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where a case is in the Phase 1 lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    /// Registered but not yet screened.
    Intake,
    /// Screening done; care or coordination in progress.
    Active,
    /// Awaiting or scheduling follow-up.
    FollowUp,
    /// No further action required (terminal).
    Closed,
}

/// Provisional Phase 1 incident taxonomy.
///
/// The final taxonomy must follow the OSCC intake forms and indicator set
/// (AGENTS.md §12 item 6); treat this list as a starting point only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentType {
    /// Sexual assault or abuse.
    SexualAssault,
    /// Domestic or family violence.
    DomesticViolence,
    /// Child abuse or neglect.
    ChildAbuse,
    /// Physical assault outside the family.
    PhysicalAssault,
    /// Human trafficking or exploitation.
    Trafficking,
    /// Anything the intake taxonomy does not cover yet.
    Other,
}

/// How quickly the case must be acted on; drives timers and alerting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// No immediate safety concern recorded.
    Low,
    /// Needs attention this shift.
    Medium,
    /// Needs attention within hours.
    High,
    /// Immediate danger; escalate now.
    Critical,
}

/// A Phase 1 case record without patient identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Case {
    /// The non-identifying case id.
    pub case_id: CaseId,
    /// Current lifecycle state.
    pub status: CaseStatus,
    /// Intake category.
    pub incident_type: IncidentType,
    /// When the incident happened, if reported.
    pub incident_at: Option<DateTime<Utc>>,
    /// When the case was registered.
    pub reported_at: DateTime<Utc>,
    /// Risk level recorded at intake (revisable, and every change is audited).
    pub risk_level: RiskLevel,
    /// Department that owns the case.
    pub owner_department: Department,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_case_id_returns_ok() {
        let id = CaseId::parse("OSCC-2026-0042").expect("valid case id");
        assert_eq!(id.as_str(), "OSCC-2026-0042");
        assert_eq!(id.to_string(), "OSCC-2026-0042");
    }

    #[test]
    fn parse_wrong_shape_returns_error() {
        assert_eq!(CaseId::parse("CASE-2026-0042"), Err(CaseIdError::Shape));
        assert_eq!(CaseId::parse("OSCC-2026"), Err(CaseIdError::Shape));
        assert_eq!(CaseId::parse("OSCC-2026-0042-1"), Err(CaseIdError::Shape));
    }

    #[test]
    fn parse_wrong_segments_returns_errors() {
        assert_eq!(CaseId::parse("OSCC-26-0042"), Err(CaseIdError::Year));
        assert_eq!(CaseId::parse("OSCC-2026-42"), Err(CaseIdError::Sequence));
    }

    #[test]
    fn serde_uses_transparent_string() {
        let id = CaseId::parse("OSCC-2026-0042").expect("valid case id");
        let json = serde_json::to_string(&id).expect("serialize");
        assert_eq!(json, "\"OSCC-2026-0042\"");
    }
}

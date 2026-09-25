//! API shapes for cases.
//!
//! Stored machine names are parsed back into domain enums here, so unknown
//! values fail closed as an internal error instead of leaking to clients.

use chrono::{DateTime, Utc};
use oscc_models::{CaseStatus, Department, IncidentType, RiskLevel};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// A queue row — deliberately identity-free (DESIGN.md case rows).
#[derive(Debug, Serialize)]
pub struct CaseSummary {
    /// Non-identifying case id.
    pub case_id: String,
    /// Lifecycle state.
    pub status: CaseStatus,
    /// Intake category.
    pub incident_type: IncidentType,
    /// Recorded risk level.
    pub risk_level: RiskLevel,
    /// Owning department.
    pub owner_department: Department,
    /// When the case was registered.
    pub reported_at: DateTime<Utc>,
}

/// Full detail with masked identity.
#[derive(Debug, Serialize)]
pub struct CaseDetail {
    /// Non-identifying case id.
    pub case_id: String,
    /// Lifecycle state.
    pub status: CaseStatus,
    /// Intake category.
    pub incident_type: IncidentType,
    /// When the incident happened, if reported.
    pub incident_at: Option<DateTime<Utc>>,
    /// When the case was registered.
    pub reported_at: DateTime<Utc>,
    /// Recorded risk level.
    pub risk_level: RiskLevel,
    /// Owning department.
    pub owner_department: Department,
    /// Masked patient identity.
    pub patient: MaskedPatient,
}

/// Identity as list and detail views may show it.
#[derive(Debug, Serialize)]
pub struct MaskedPatient {
    /// Masked hospital number.
    pub hn: String,
    /// Masked national ID.
    pub cid: String,
    /// Masked name.
    pub name: String,
    /// When the snapshot was taken.
    pub snapshot_at: DateTime<Utc>,
}

/// Reveal payload: the reason is mandatory and recorded in the audit trail.
#[derive(Debug, Deserialize)]
pub struct RevealRequest {
    /// Why the identity is being revealed.
    pub reason: String,
}

/// Full identity, returned only by the audited reveal path.
#[derive(Debug, Serialize)]
pub struct RevealedPatient {
    /// Hospital number snapshot.
    pub hn: String,
    /// National ID snapshot.
    pub cid: String,
    /// Name snapshot.
    pub name: String,
    /// When the snapshot was taken.
    pub snapshot_at: DateTime<Utc>,
}

/// Intake payload. Exactly one of `hn`/`cid` selects the patient.
#[derive(Debug, Deserialize)]
pub struct CreateCaseRequest {
    /// Hospital number, when known.
    pub hn: Option<String>,
    /// National ID, when known.
    pub cid: Option<String>,
    /// Intake category.
    pub incident_type: IncidentType,
    /// When the incident happened, if reported.
    pub incident_at: Option<DateTime<Utc>>,
    /// Recorded risk level.
    pub risk_level: RiskLevel,
    /// Owning department.
    pub owner_department: Department,
}

/// The one valid patient selector from an intake request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatientKey {
    /// Hospital number.
    Hn(String),
    /// National ID.
    Cid(String),
}

impl CreateCaseRequest {
    /// Validates that exactly one of `hn`/`cid` is present and non-empty.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::BadRequest`] when both or neither are given.
    pub fn lookup_key(&self) -> Result<PatientKey, AppError> {
        let hn = self.hn.as_deref().map(str::trim).filter(|v| !v.is_empty());
        let cid = self.cid.as_deref().map(str::trim).filter(|v| !v.is_empty());
        match (hn, cid) {
            (Some(hn), None) => Ok(PatientKey::Hn(hn.to_string())),
            (None, Some(cid)) => Ok(PatientKey::Cid(cid.to_string())),
            _ => Err(AppError::BadRequest(
                "provide exactly one of hospital number or national ID",
            )),
        }
    }
}

/// Parses a stored status.
///
/// # Errors
///
/// Returns [`AppError::Internal`] for unknown values.
pub fn parse_status(raw: &str) -> Result<CaseStatus, AppError> {
    CaseStatus::parse(raw).ok_or(AppError::Internal)
}

/// Parses a stored incident type.
///
/// # Errors
///
/// Returns [`AppError::Internal`] for unknown values.
pub fn parse_incident(raw: &str) -> Result<IncidentType, AppError> {
    IncidentType::parse(raw).ok_or(AppError::Internal)
}

/// Parses a stored risk level.
///
/// # Errors
///
/// Returns [`AppError::Internal`] for unknown values.
pub fn parse_risk(raw: &str) -> Result<RiskLevel, AppError> {
    RiskLevel::parse(raw).ok_or(AppError::Internal)
}

/// Parses a stored department.
///
/// # Errors
///
/// Returns [`AppError::Internal`] for unknown values.
pub fn parse_department(raw: &str) -> Result<Department, AppError> {
    Department::parse(raw).ok_or(AppError::Internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(hn: Option<&str>, cid: Option<&str>) -> CreateCaseRequest {
        CreateCaseRequest {
            hn: hn.map(str::to_string),
            cid: cid.map(str::to_string),
            incident_type: IncidentType::Other,
            incident_at: None,
            risk_level: RiskLevel::Low,
            owner_department: Department::Emergency,
        }
    }

    #[test]
    fn lookup_key_accepts_exactly_one_selector() {
        assert_eq!(
            request(Some("12345"), None).lookup_key().ok(),
            Some(PatientKey::Hn("12345".to_string()))
        );
        assert_eq!(
            request(None, Some("1101701234567")).lookup_key().ok(),
            Some(PatientKey::Cid("1101701234567".to_string()))
        );
    }

    #[test]
    fn lookup_key_rejects_both_or_neither() {
        assert!(
            request(Some("12345"), Some("1101701234567"))
                .lookup_key()
                .is_err()
        );
        assert!(request(None, None).lookup_key().is_err());
        assert!(request(Some("   "), None).lookup_key().is_err());
    }

    #[test]
    fn stored_enums_parse_and_unknown_values_fail_closed() {
        assert_eq!(parse_status("intake").ok(), Some(CaseStatus::Intake));
        assert_eq!(parse_risk("critical").ok(), Some(RiskLevel::Critical));
        assert!(parse_status("nope").is_err());
        assert!(parse_department("nope").is_err());
    }
}

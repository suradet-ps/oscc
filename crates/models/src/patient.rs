//! Patient identity snapshot pulled from HOSxP at intake.

use std::fmt::{self, Debug, Formatter};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cid::Cid;

/// A hospital number (HN) as issued by HOSxP.
///
/// The HN is PII; its [`Debug`] output is redacted. The exact HN format
/// varies per hospital (AGENTS.md §12 item 6), so this type only enforces
/// non-emptiness and a sane upper bound.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hn(String);

/// Why a string is not a usable hospital number.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HnError {
    /// The input is empty after trimming.
    #[error("hospital number must not be empty")]
    Empty,
    /// The input is longer than the accepted maximum.
    #[error("hospital number must be at most {max} characters")]
    TooLong { max: usize },
}

impl Hn {
    /// The longest accepted HN, generous enough for every known HOSxP format.
    pub const MAX_LEN: usize = 20;

    /// Validates `raw` as a non-empty HN.
    ///
    /// # Errors
    ///
    /// Returns [`HnError`] when the input is empty or too long.
    pub fn parse(raw: &str) -> Result<Self, HnError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(HnError::Empty);
        }
        if trimmed.chars().count() > Self::MAX_LEN {
            return Err(HnError::TooLong { max: Self::MAX_LEN });
        }
        Ok(Self(trimmed.to_string()))
    }

    /// The HN as text — only for the audited reveal path and storage.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl Debug for Hn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Hn(***)")
    }
}

/// The patient-identifying part of a case, snapshotted from HOSxP.
///
/// Snapshot semantics (AGENTS.md §3): HOSxP keeps changing, but the record
/// of who the case was about must not. Every field is PII; the custom
/// [`Debug`] keeps the struct safe to pass through error types and logs.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatientLink {
    /// Hospital number at intake.
    pub hn: Hn,
    /// National ID at intake; display only through [`Cid::masked`].
    pub cid: Cid,
    /// Name as HOSxP had it at intake.
    pub name_snapshot: String,
    /// When the snapshot was taken.
    pub snapshot_at: DateTime<Utc>,
}

impl Debug for PatientLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PatientLink")
            .field("hn", &"***")
            .field("cid", &"***")
            .field("name_snapshot", &"***")
            .field("snapshot_at", &self.snapshot_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn cid() -> Cid {
        Cid::parse("1101701234567").expect("valid test ID")
    }

    fn snapshot_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 25, 8, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    #[test]
    fn hn_parse_trims_and_accepts() {
        let hn = Hn::parse(" 12345 ").expect("valid HN");
        assert_eq!(hn.expose(), "12345");
    }

    #[test]
    fn hn_parse_empty_returns_error() {
        assert_eq!(Hn::parse("   "), Err(HnError::Empty));
    }

    #[test]
    fn hn_parse_too_long_returns_error() {
        let long = "1".repeat(Hn::MAX_LEN + 1);
        assert_eq!(Hn::parse(&long), Err(HnError::TooLong { max: Hn::MAX_LEN }));
    }

    #[test]
    fn hn_debug_redacts_the_value() {
        let hn = Hn::parse("12345").expect("valid HN");
        assert_eq!(format!("{hn:?}"), "Hn(***)");
    }

    #[test]
    fn patient_link_debug_contains_no_pii() {
        let link = PatientLink {
            hn: Hn::parse("12345").expect("valid HN"),
            cid: cid(),
            name_snapshot: "สมชาย ทดสอบ".to_string(),
            snapshot_at: snapshot_at(),
        };
        let debug = format!("{link:?}");
        assert!(!debug.contains("12345"));
        assert!(!debug.contains("1101701234567"));
        assert!(!debug.contains("สมชาย"));
    }
}

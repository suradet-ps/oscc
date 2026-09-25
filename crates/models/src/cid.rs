//! Thai national ID (เลขประจำตัวประชาชน) with mandatory masking support.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A validated 13-digit Thai national ID.
///
/// The full value is PII: it must never appear in logs, error messages, or
/// `Debug` output. Display through [`Cid::masked`] everywhere outside the
/// audited reveal flow (AGENTS.md §2, DESIGN.md identity rules).
///
/// # Examples
///
/// ```
/// use oscc_models::Cid;
///
/// let cid = Cid::parse("1101701234567")?;
/// assert_eq!(cid.masked(), "1-XXXX-XXXXX-XX-7");
/// # Ok::<(), oscc_models::CidError>(())
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cid(String);

/// Why a string is not a valid national ID.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CidError {
    /// The input does not contain exactly 13 characters.
    #[error("national ID must be 13 digits, got {len} characters")]
    Length { len: usize },
    /// The input contains a non-digit character.
    #[error("national ID must contain digits only")]
    NonDigit,
}

impl Cid {
    /// Validates `raw` as 13 ASCII digits.
    ///
    /// # Errors
    ///
    /// Returns [`CidError::Length`] or [`CidError::NonDigit`] when the input
    /// is not a 13-digit string.
    pub fn parse(raw: &str) -> Result<Self, CidError> {
        let len = raw.chars().count();
        if len != 13 {
            return Err(CidError::Length { len });
        }
        if !raw.chars().all(|c| c.is_ascii_digit()) {
            return Err(CidError::NonDigit);
        }
        Ok(Self(raw.to_string()))
    }

    /// The full value — only for the audited reveal path and storage.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// The masked form `1-XXXX-XXXXX-XX-3`, safe for screens and logs.
    pub fn masked(&self) -> String {
        let first = self.0.chars().next().unwrap_or('0');
        let last = self.0.chars().last().unwrap_or('0');
        format!("{first}-XXXX-XXXXX-XX-{last}")
    }
}

impl fmt::Debug for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Cid(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_id_returns_ok() {
        assert!(Cid::parse("1101701234567").is_ok());
    }

    #[test]
    fn parse_wrong_length_returns_error() {
        assert_eq!(Cid::parse("11017012345"), Err(CidError::Length { len: 11 }));
        assert_eq!(
            Cid::parse("11017012345678"),
            Err(CidError::Length { len: 14 })
        );
    }

    #[test]
    fn parse_non_digit_returns_error() {
        assert_eq!(Cid::parse("110170123456x"), Err(CidError::NonDigit));
    }

    #[test]
    fn masked_hides_middle_digits() {
        let cid = Cid::parse("1101701234567").expect("valid test ID");
        assert_eq!(cid.masked(), "1-XXXX-XXXXX-XX-7");
    }

    #[test]
    fn debug_output_redacts_the_value() {
        let cid = Cid::parse("1101701234567").expect("valid test ID");
        let debug = format!("{cid:?}");
        assert!(!debug.contains("1101701234567"));
        assert_eq!(debug, "Cid(***)");
    }

    #[test]
    fn serde_round_trip_keeps_value() {
        let cid = Cid::parse("1101701234567").expect("valid test ID");
        let json = serde_json::to_string(&cid).expect("serialize");
        assert_eq!(json, "\"1101701234567\"");
        let back: Cid = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, cid);
    }
}

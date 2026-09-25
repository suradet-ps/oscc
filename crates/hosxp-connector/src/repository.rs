//! Read-only patient lookups against HOSxP.
//!
//! Values are validated and escaped before they become query parameters;
//! masking and permission checks happen at the API boundary, not here.

use chrono::NaiveDate;
use sqlx::MySqlPool;

use crate::error::Error;
use crate::queries;
use crate::readonly_guard::assert_read_only;

/// What the caller is looking for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatientQuery {
    /// Exact hospital number.
    Hn(String),
    /// Exact 13-digit national ID.
    Cid(String),
    /// Name fragment, matched as a prefix.
    NamePrefix(String),
}

/// One patient row as HOSxP stores it; raw values, never logged.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct PatientRow {
    /// Hospital number.
    pub hn: String,
    /// National ID, absent when HOSxP has none on file.
    pub cid: Option<String>,
    /// Full name in Thai.
    pub full_name_th: String,
    /// Birth date, absent when unknown.
    pub birth_date: Option<NaiveDate>,
    /// Sex as HOSxP records it.
    pub sex: Option<String>,
}

/// The longest accepted HN; generous for every known HOSxP pattern.
const MAX_HN_LEN: usize = 20;

/// The longest accepted name fragment.
const MAX_NAME_LEN: usize = 50;

/// Looks up patients and returns HOSxP's current state.
///
/// The caller snapshots what it needs; this function never writes.
///
/// # Errors
///
/// Returns [`Error::InvalidQuery`] for malformed input and
/// [`Error::Database`]/[`Error::Guard`] when the statement fails.
pub async fn find_patients(
    pool: &MySqlPool,
    query: &PatientQuery,
) -> Result<Vec<PatientRow>, Error> {
    let (sql, params) = statement_for(query)?;
    assert_read_only(sql)?;

    let mut statement = sqlx::query_as::<_, PatientRow>(sql);
    for param in &params {
        statement = statement.bind(param);
    }
    statement.fetch_all(pool).await.map_err(Error::Database)
}

/// Builds the statement and its parameter list for `query`.
///
/// Split out from [`find_patients`] so validation and escaping are testable
/// without a database.
fn statement_for(query: &PatientQuery) -> Result<(&'static str, Vec<String>), Error> {
    match query {
        PatientQuery::Hn(raw) => {
            let hn = raw.trim();
            if hn.is_empty() {
                return Err(Error::InvalidQuery("hospital number must not be empty"));
            }
            if hn.chars().count() > MAX_HN_LEN {
                return Err(Error::InvalidQuery("hospital number is too long"));
            }
            Ok((queries::PATIENT_BY_HN, vec![hn.to_string()]))
        }
        PatientQuery::Cid(raw) => {
            let cid = raw.trim();
            if cid.chars().count() != 13 || !cid.chars().all(|c| c.is_ascii_digit()) {
                return Err(Error::InvalidQuery("national ID must be exactly 13 digits"));
            }
            Ok((queries::PATIENT_BY_CID, vec![cid.to_string()]))
        }
        PatientQuery::NamePrefix(raw) => {
            let term = raw.trim();
            if term.is_empty() {
                return Err(Error::InvalidQuery("name fragment must not be empty"));
            }
            if term.chars().count() > MAX_NAME_LEN {
                return Err(Error::InvalidQuery("name fragment is too long"));
            }
            let prefix = format!("{}%", escape_like(term));
            Ok((
                queries::PATIENT_BY_NAME_PREFIX,
                vec![prefix.clone(), prefix.clone(), prefix],
            ))
        }
    }
}

/// Escapes MySQL `LIKE` wildcards so user input matches literally. The
/// backslash is MySQL's default escape character, so no `ESCAPE` clause is
/// needed.
fn escape_like(term: &str) -> String {
    let mut escaped = String::with_capacity(term.len());
    for c in term.chars() {
        match c {
            '\\' | '%' | '_' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hn_is_trimmed_and_accepted() {
        let (sql, params) = statement_for(&PatientQuery::Hn(" 12345 ".into())).expect("valid HN");
        assert_eq!(sql, queries::PATIENT_BY_HN);
        assert_eq!(params, vec!["12345"]);
    }

    #[test]
    fn empty_hn_is_rejected() {
        assert!(matches!(
            statement_for(&PatientQuery::Hn("   ".into())),
            Err(Error::InvalidQuery(_))
        ));
    }

    #[test]
    fn oversized_hn_is_rejected() {
        let long = "1".repeat(MAX_HN_LEN + 1);
        assert!(matches!(
            statement_for(&PatientQuery::Hn(long)),
            Err(Error::InvalidQuery(_))
        ));
    }

    #[test]
    fn cid_must_be_thirteen_digits() {
        assert!(statement_for(&PatientQuery::Cid("1101701234567".into())).is_ok());
        assert!(matches!(
            statement_for(&PatientQuery::Cid("11017012345".into())),
            Err(Error::InvalidQuery(_))
        ));
        assert!(matches!(
            statement_for(&PatientQuery::Cid("110170123456x".into())),
            Err(Error::InvalidQuery(_))
        ));
    }

    #[test]
    fn name_prefix_adds_one_wildcard_and_escapes_input_wildcards() {
        let (sql, params) =
            statement_for(&PatientQuery::NamePrefix("สม%ชาย".into())).expect("valid name");
        assert_eq!(sql, queries::PATIENT_BY_NAME_PREFIX);
        assert_eq!(params, vec!["สม\\%ชาย%"; 3]);
    }

    #[test]
    fn name_prefix_escapes_underscore_and_backslash() {
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("a\\b"), "a\\\\b");
    }

    #[test]
    fn empty_name_fragment_is_rejected() {
        assert!(matches!(
            statement_for(&PatientQuery::NamePrefix(" ".into())),
            Err(Error::InvalidQuery(_))
        ));
    }
}

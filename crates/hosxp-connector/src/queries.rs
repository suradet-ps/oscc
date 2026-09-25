//! Every SQL statement this crate may execute — compile-time constants only.
//!
//! Runtime values are always bound as parameters, never interpolated
//! (AGENTS.md §2 rule 1). Every statement must pass [`assert_read_only`] —
//! enforced by the test at the bottom of this module.
//!
//! HOSxP table and column names are provisional until confirmed against the
//! live instance (AGENTS.md §6); each statement carries a
//! `SCHEMA-UNVERIFIED` note until then.
//!
//! [`assert_read_only`]: crate::readonly_guard::assert_read_only

/// Exact hospital-number lookup.
///
/// // SCHEMA-UNVERIFIED: `patient.pname/fname/lname/birthday/cid/sex` follow
/// // standard HOSxP v3/v4 conventions; confirm with `SHOW COLUMNS` before
/// // the pilot.
pub const PATIENT_BY_HN: &str = "SELECT hn, cid, CONCAT_WS(' ', pname, fname, lname) AS full_name_th, \
     birthday AS birth_date, sex \
     FROM patient WHERE hn = ? LIMIT 20";

/// Exact national-ID lookup.
///
/// // SCHEMA-UNVERIFIED: same `patient` surface as [`PATIENT_BY_HN`].
pub const PATIENT_BY_CID: &str = "SELECT hn, cid, CONCAT_WS(' ', pname, fname, lname) AS full_name_th, \
     birthday AS birth_date, sex \
     FROM patient WHERE cid = ? LIMIT 20";

/// Name lookup by prefix, so existing indexes on `fname`/`lname` can be used.
///
/// // SCHEMA-UNVERIFIED: same `patient` surface as [`PATIENT_BY_HN`].
pub const PATIENT_BY_NAME_PREFIX: &str = "SELECT hn, cid, CONCAT_WS(' ', pname, fname, lname) AS full_name_th, \
     birthday AS birth_date, sex \
     FROM patient \
     WHERE fname LIKE ? OR lname LIKE ? OR CONCAT_WS(' ', pname, fname, lname) LIKE ? \
     LIMIT 20";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readonly_guard::assert_read_only;

    /// Every statement the crate can execute. Add new constants here.
    const ALL_STATEMENTS: [&str; 3] = [PATIENT_BY_HN, PATIENT_BY_CID, PATIENT_BY_NAME_PREFIX];

    /// The table surface documented in docs/data-map.md — the DBA grants are
    /// written against it.
    const DOCUMENTED_TABLES: [&str; 1] = ["patient"];

    #[test]
    fn all_statements_are_read_only() {
        for sql in ALL_STATEMENTS {
            assert!(assert_read_only(sql).is_ok(), "must be SELECT: {sql}");
        }
    }

    /// Identifiers immediately following a standalone `FROM` keyword — the
    /// tables a statement reads.
    fn referenced_tables(sql: &str) -> Vec<String> {
        let lower = sql.to_ascii_lowercase();
        let bytes = lower.as_bytes();
        let mut tables = Vec::new();
        let mut start = 0;
        while let Some(rel) = lower[start..].find("from") {
            let at = start + rel;
            let end = at + 4;
            let word_bounded = (at == 0 || !bytes[at - 1].is_ascii_alphanumeric())
                && (end == bytes.len() || !bytes[end].is_ascii_alphanumeric());
            if word_bounded {
                let ident: String = lower[end..]
                    .chars()
                    .skip_while(|c| c.is_whitespace())
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if !ident.is_empty() {
                    tables.push(ident);
                }
            }
            start = end;
        }
        tables
    }

    #[test]
    fn every_referenced_table_is_on_the_documented_access_surface() {
        for sql in ALL_STATEMENTS {
            for table in referenced_tables(sql) {
                assert!(
                    DOCUMENTED_TABLES.contains(&table.as_str()),
                    "table `{table}` is not on the documented HOSxP access surface \
                     (docs/data-map.md) — update that doc and the DBA grants \
                     in the same change"
                );
            }
        }
    }
}

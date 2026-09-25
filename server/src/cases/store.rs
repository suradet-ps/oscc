//! Case storage. The only writer of case rows (AGENTS.md §3).

use chrono::{DateTime, Utc};
use oscc_models::{CaseStatus, Department, IncidentType, RiskLevel};
use sqlx::PgPool;

use super::dto::{self, CaseDetail, CaseSummary, MaskedPatient};
use crate::error::AppError;

/// A stored case without identity.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CaseRow {
    /// Surrogate key.
    pub id: i64,
    /// Non-identifying case id.
    pub case_id: String,
    /// Stored status machine name.
    pub status: String,
    /// Stored incident machine name.
    pub incident_type: String,
    /// When the incident happened, if reported.
    pub incident_at: Option<DateTime<Utc>>,
    /// When the case was registered.
    pub reported_at: DateTime<Utc>,
    /// Stored risk machine name.
    pub risk_level: String,
    /// Stored department machine name.
    pub owner_department: String,
}

/// A stored case joined with its patient snapshot.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CaseDetailRow {
    /// Surrogate key.
    pub id: i64,
    /// Non-identifying case id.
    pub case_id: String,
    /// Stored status machine name.
    pub status: String,
    /// Stored incident machine name.
    pub incident_type: String,
    /// When the incident happened, if reported.
    pub incident_at: Option<DateTime<Utc>>,
    /// When the case was registered.
    pub reported_at: DateTime<Utc>,
    /// Stored risk machine name.
    pub risk_level: String,
    /// Stored department machine name.
    pub owner_department: String,
    /// Hospital number snapshot.
    pub hn: String,
    /// National ID snapshot.
    pub cid: String,
    /// Name snapshot.
    pub name_snapshot: String,
    /// When the snapshot was taken.
    pub snapshot_at: DateTime<Utc>,
}

/// Everything needed to insert a case and its snapshot atomically.
pub struct NewCase<'a> {
    /// Hospital number snapshot.
    pub hn: &'a str,
    /// National ID snapshot.
    pub cid: &'a str,
    /// Name snapshot.
    pub name_snapshot: &'a str,
    /// When the snapshot was taken.
    pub snapshot_at: DateTime<Utc>,
    /// Intake category.
    pub incident_type: IncidentType,
    /// When the incident happened, if reported.
    pub incident_at: Option<DateTime<Utc>>,
    /// Recorded risk level.
    pub risk_level: RiskLevel,
    /// Owning department.
    pub owner_department: Department,
    /// Account that registered the case.
    pub created_by: i64,
}

impl CaseRow {
    /// Converts to the identity-free queue shape.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] for unknown stored values.
    pub fn summary(&self) -> Result<CaseSummary, AppError> {
        Ok(CaseSummary {
            case_id: self.case_id.clone(),
            status: dto::parse_status(&self.status)?,
            incident_type: dto::parse_incident(&self.incident_type)?,
            risk_level: dto::parse_risk(&self.risk_level)?,
            owner_department: dto::parse_department(&self.owner_department)?,
            reported_at: self.reported_at,
        })
    }
}

impl CaseDetailRow {
    /// Converts to the masked detail shape.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] for unknown stored values.
    pub fn detail(&self) -> Result<CaseDetail, AppError> {
        Ok(CaseDetail {
            case_id: self.case_id.clone(),
            status: dto::parse_status(&self.status)?,
            incident_type: dto::parse_incident(&self.incident_type)?,
            incident_at: self.incident_at,
            reported_at: self.reported_at,
            risk_level: dto::parse_risk(&self.risk_level)?,
            owner_department: dto::parse_department(&self.owner_department)?,
            patient: MaskedPatient {
                hn: super::mask::mask_hn(&self.hn),
                cid: oscc_models::Cid::parse(&self.cid)
                    .map(|cid| cid.masked())
                    .unwrap_or_else(|_| "*****".to_string()),
                name: super::mask::mask_name(&self.name_snapshot),
                snapshot_at: self.snapshot_at,
            },
        })
    }
}

/// Inserts a case and its patient snapshot in one transaction.
///
/// The case id comes from `case_number_seq`, formatted `OSCC-<year>-<nnnn>`.
///
/// # Errors
///
/// Returns [`AppError::Internal`] when the transaction fails.
pub async fn insert_case(pool: &PgPool, new: &NewCase<'_>) -> Result<CaseRow, AppError> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as::<_, CaseRow>(
        "INSERT INTO cases \
         (case_id, status, incident_type, incident_at, risk_level, owner_department, created_by) \
         VALUES ( \
             'OSCC-' || to_char(now(), 'YYYY') || '-' || \
             lpad(nextval('case_number_seq')::text, 4, '0'), \
             'intake', $1, $2, $3, $4, $5) \
         RETURNING id, case_id, status, incident_type, incident_at, reported_at, \
                   risk_level, owner_department",
    )
    .bind(new.incident_type.as_str())
    .bind(new.incident_at)
    .bind(new.risk_level.as_str())
    .bind(new.owner_department.as_str())
    .bind(new.created_by)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO patient_links (case_pk, hn, cid, name_snapshot, snapshot_at) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(row.id)
    .bind(new.hn)
    .bind(new.cid)
    .bind(new.name_snapshot)
    .bind(new.snapshot_at)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(row)
}

/// Lists cases for the queue, newest first.
///
/// # Errors
///
/// Returns [`AppError::Internal`] when the query fails.
pub async fn list_cases(
    pool: &PgPool,
    status: Option<CaseStatus>,
    limit: i64,
) -> Result<Vec<CaseRow>, AppError> {
    let status = status.map(|value| value.as_str().to_string());
    let rows = sqlx::query_as::<_, CaseRow>(
        "SELECT id, case_id, status, incident_type, incident_at, reported_at, \
                risk_level, owner_department \
         FROM cases \
         WHERE ($1::text IS NULL OR status = $1) \
         ORDER BY reported_at DESC, id DESC LIMIT $2",
    )
    .bind(status)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Finds one case with its patient snapshot.
///
/// # Errors
///
/// Returns [`AppError::Internal`] when the query fails.
pub async fn find_case(pool: &PgPool, case_id: &str) -> Result<Option<CaseDetailRow>, AppError> {
    let row = sqlx::query_as::<_, CaseDetailRow>(
        "SELECT c.id, c.case_id, c.status, c.incident_type, c.incident_at, c.reported_at, \
                c.risk_level, c.owner_department, p.hn, p.cid, p.name_snapshot, p.snapshot_at \
         FROM cases c JOIN patient_links p ON p.case_pk = c.id \
         WHERE c.case_id = $1",
    )
    .bind(case_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

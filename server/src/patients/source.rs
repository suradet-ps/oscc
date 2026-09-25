//! The patient-source abstraction.
//!
//! Production reads HOSxP through the connector; tests provide their own
//! implementation, so the case flow is testable without a live HOSxP.

use std::sync::Arc;

use async_trait::async_trait;
use oscc_hosxp_connector::{Error as ConnectorError, PatientQuery, PatientRow, find_patients};
use sqlx::MySqlPool;
use thiserror::Error;

/// Why a patient lookup failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PatientSourceError {
    /// HOSxP is not reachable or not configured.
    #[error("HOSxP is not reachable")]
    Unavailable,
    /// The query itself was malformed.
    #[error("patient query is invalid")]
    Invalid,
}

/// A read-only source of patient demographics.
#[async_trait]
pub trait PatientSource: Send + Sync {
    /// Finds patients matching `query`, newest HOSxP state.
    async fn find(&self, query: PatientQuery) -> Result<Vec<PatientRow>, PatientSourceError>;
}

/// Production source: the HOSxP connector over its own MySQL pool.
pub struct HosxpPatientSource {
    pool: MySqlPool,
}

impl HosxpPatientSource {
    /// Wraps a connector pool.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PatientSource for HosxpPatientSource {
    async fn find(&self, query: PatientQuery) -> Result<Vec<PatientRow>, PatientSourceError> {
        find_patients(&self.pool, &query)
            .await
            .map_err(|err| match err {
                ConnectorError::InvalidQuery(_) => PatientSourceError::Invalid,
                _ => PatientSourceError::Unavailable,
            })
    }
}

/// A shared patient source, stored in the API state.
pub type SharedPatientSource = Arc<dyn PatientSource>;

/// Development-only source with one canned patient, enabled with
/// `OSCC_PATIENT_SOURCE=fake` on machines without HOSxP. It exists so the
/// intake flow can be exercised locally; it must never be enabled in a real
/// deployment (AGENTS.md §2 rule 6: synthetic data only outside production).
pub struct FakeDevSource;

#[async_trait]
impl PatientSource for FakeDevSource {
    async fn find(&self, _query: PatientQuery) -> Result<Vec<PatientRow>, PatientSourceError> {
        Ok(vec![PatientRow {
            hn: "12345".to_string(),
            cid: Some("1101701234567".to_string()),
            full_name_th: "ผู้ป่วย ทดสอบ".to_string(),
            birth_date: None,
            sex: Some("ชาย".to_string()),
        }])
    }
}

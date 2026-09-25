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

//! Patient sources and lookup routes.
//!
//! Production reads HOSxP through the connector; a development-only fake
//! source exists for machines without HOSxP. Masking happens in
//! [`routes`], never in the database.

pub mod routes;
pub mod source;

pub use source::{
    FakeDevSource, HosxpPatientSource, PatientSource, PatientSourceError, SharedPatientSource,
};

use crate::error::AppError;

/// Maps a lookup failure to the API error surface.
pub(crate) fn error_to_app(err: PatientSourceError) -> AppError {
    match err {
        PatientSourceError::Unavailable => AppError::Unavailable("HOSxP is not reachable"),
        PatientSourceError::Invalid => AppError::BadRequest("patient query is invalid"),
    }
}

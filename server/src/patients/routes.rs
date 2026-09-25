//! `/api/v1/patients` routes.

use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use chrono::NaiveDate;
use oscc_core::Permission;
use oscc_hosxp_connector::{PatientQuery, PatientRow};
use oscc_models::{AuditAction, Cid};
use serde::{Deserialize, Serialize};

use super::error_to_app;
use crate::AppState;
use crate::audit::{self, AuditEvent};
use crate::auth::AuthUser;
use crate::cases::mask::{mask_hn, mask_name};
use crate::error::AppError;

/// Builds the patient router (mounted under `/api/v1/patients`).
pub fn routes() -> Router<AppState> {
    Router::new().route("/lookup", get(lookup))
}

/// Query parameters: exactly one of `hn`/`cid`.
#[derive(Debug, Deserialize)]
struct LookupParams {
    /// Hospital number.
    hn: Option<String>,
    /// National ID.
    cid: Option<String>,
}

/// A candidate as the intake screen may show it: masked.
#[derive(Debug, Serialize)]
struct MaskedCandidate {
    hn: String,
    cid: String,
    name: String,
    birth_date: Option<NaiveDate>,
    sex: Option<String>,
}

/// Looks up patients in HOSxP and returns masked candidates.
///
/// The full identity never travels on this path; intake re-reads HOSxP to
/// snapshot it, and reveal is a separate audited action. The lookup itself
/// is audited with the query kind only — never the search value.
async fn lookup(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<LookupParams>,
) -> Result<Json<Vec<MaskedCandidate>>, AppError> {
    user.require(Permission::ViewCase)?;
    let pool = state.pool()?;

    let hn = params
        .hn
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let cid = params
        .cid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let (query, kind) = match (hn, cid) {
        (Some(hn), None) => (PatientQuery::Hn(hn.to_string()), "hn"),
        (None, Some(cid)) => (PatientQuery::Cid(cid.to_string()), "cid"),
        _ => {
            return Err(AppError::BadRequest("provide exactly one of hn or cid"));
        }
    };

    let rows = state.patients()?.find(query).await.map_err(error_to_app)?;

    audit::append(
        pool,
        &AuditEvent {
            action: AuditAction::PatientLookedUp,
            actor: &user.username,
            actor_role: Some(user.role),
            case_id: None,
            reason: None,
            detail: serde_json::json!({ "query_kind": kind }),
        },
    )
    .await?;

    Ok(Json(rows.iter().map(mask_candidate).collect()))
}

/// Masks one HOSxP row for display.
fn mask_candidate(row: &PatientRow) -> MaskedCandidate {
    MaskedCandidate {
        hn: mask_hn(&row.hn),
        cid: row
            .cid
            .as_deref()
            .and_then(|raw| Cid::parse(raw).ok())
            .map(|cid| cid.masked())
            .unwrap_or_else(|| "*****".to_string()),
        name: mask_name(&row.full_name_th),
        birth_date: row.birth_date,
        sex: row.sex.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_masking_hides_identity() {
        let row = PatientRow {
            hn: "12345".to_string(),
            cid: Some("1101701234567".to_string()),
            full_name_th: "สมชาย ทดสอบ".to_string(),
            birth_date: None,
            sex: Some("ชาย".to_string()),
        };
        let masked = mask_candidate(&row);
        assert_eq!(masked.hn, "12****45");
        assert_eq!(masked.cid, "1-XXXX-XXXXX-XX-7");
        assert_eq!(masked.name, "สม*****");
    }

    #[test]
    fn candidate_without_valid_cid_masks_entirely() {
        let row = PatientRow {
            hn: "1".to_string(),
            cid: None,
            full_name_th: String::new(),
            birth_date: None,
            sex: None,
        };
        let masked = mask_candidate(&row);
        assert_eq!(masked.cid, "*****");
        assert_eq!(masked.hn, "****");
        assert_eq!(masked.name, "*****");
    }
}

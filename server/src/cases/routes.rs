//! `/api/v1/cases` routes: intake, queue, and detail.
//!
//! Every response masks identity; every action lands in the audit trail
//! (AGENTS.md §2 rule 4). Handlers stay thin: permission check, domain
//! call, shape the response.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use oscc_core::Permission;
use oscc_models::{AuditAction, Cid, Hn};
use serde::Deserialize;

use super::dto::{CaseDetail, CaseSummary, CreateCaseRequest, PatientKey};
use super::store::{self, NewCase};
use crate::AppState;
use crate::audit::{self, AuditEvent};
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::patients::PatientSourceError;

/// Builds the case router (mounted under `/api/v1/cases`).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_case).get(list_cases))
        .route("/{case_id}", get(get_case))
}

/// Registers a case for a patient found in HOSxP, snapshotting the
/// demographics HOSxP had at that moment.
async fn create_case(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<CreateCaseRequest>,
) -> Result<(StatusCode, Json<CaseDetail>), AppError> {
    user.require(Permission::CreateCase)?;
    let pool = state.pool()?;
    let key = request.lookup_key()?;

    let rows = state
        .patients()?
        .find(key.to_query())
        .await
        .map_err(patient_source_error)?;
    let row = rows
        .into_iter()
        .next()
        .ok_or(AppError::NotFound("patient not found in HOSxP"))?;

    let cid_raw = row
        .cid
        .as_deref()
        .ok_or(AppError::BadRequest("patient has no national ID on file"))?;
    let cid =
        Cid::parse(cid_raw).map_err(|_| AppError::BadRequest("patient national ID is invalid"))?;
    let hn = Hn::parse(&row.hn).map_err(|_| AppError::Internal)?;

    let new = NewCase {
        hn: hn.expose(),
        cid: cid.expose(),
        name_snapshot: &row.full_name_th,
        snapshot_at: Utc::now(),
        incident_type: request.incident_type,
        incident_at: request.incident_at,
        risk_level: request.risk_level,
        owner_department: request.owner_department,
        created_by: user.user_id,
    };
    let case = store::insert_case(pool, &new).await?;

    audit::append(
        pool,
        &AuditEvent {
            action: AuditAction::CaseOpened,
            actor: &user.username,
            actor_role: Some(user.role),
            case_id: Some(&case.case_id),
            reason: None,
            detail: serde_json::json!({
                "incident_type": request.incident_type.as_str(),
                "risk_level": request.risk_level.as_str(),
            }),
        },
    )
    .await?;

    let detail = store::find_case(pool, &case.case_id)
        .await?
        .ok_or(AppError::Internal)?;
    Ok((StatusCode::CREATED, Json(detail.detail()?)))
}

/// Query parameters for the queue.
#[derive(Debug, Deserialize)]
struct ListParams {
    /// Optional status filter, machine name.
    status: Option<String>,
    /// Page size, clamped to 1..=200.
    limit: Option<i64>,
}

/// Lists the case queue, newest first, identity-free.
async fn list_cases(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<CaseSummary>>, AppError> {
    user.require(Permission::ViewCase)?;
    let pool = state.pool()?;

    let status = match params.status.as_deref() {
        Some(raw) => Some(super::dto::parse_status(raw)?),
        None => None,
    };
    let limit = params.limit.unwrap_or(50).clamp(1, 200);

    let rows = store::list_cases(pool, status, limit).await?;
    let summaries = rows
        .iter()
        .map(store::CaseRow::summary)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(summaries))
}

/// Loads one case with its masked identity and records the view.
async fn get_case(
    State(state): State<AppState>,
    user: AuthUser,
    Path(case_id): Path<String>,
) -> Result<Json<CaseDetail>, AppError> {
    user.require(Permission::ViewCase)?;
    let pool = state.pool()?;

    let row = store::find_case(pool, &case_id)
        .await?
        .ok_or(AppError::NotFound("case not found"))?;

    audit::append(
        pool,
        &AuditEvent {
            action: AuditAction::CaseViewed,
            actor: &user.username,
            actor_role: Some(user.role),
            case_id: Some(&row.case_id),
            reason: None,
            detail: serde_json::json!({}),
        },
    )
    .await?;

    Ok(Json(row.detail()?))
}

impl PatientKey {
    /// The connector query for this key.
    pub fn to_query(&self) -> oscc_hosxp_connector::PatientQuery {
        match self {
            Self::Hn(hn) => oscc_hosxp_connector::PatientQuery::Hn(hn.clone()),
            Self::Cid(cid) => oscc_hosxp_connector::PatientQuery::Cid(cid.clone()),
        }
    }
}

/// Maps a lookup failure to the API error surface.
fn patient_source_error(err: PatientSourceError) -> AppError {
    match err {
        PatientSourceError::Unavailable => AppError::Unavailable("HOSxP is not reachable"),
        PatientSourceError::Invalid => AppError::BadRequest("patient query is invalid"),
    }
}

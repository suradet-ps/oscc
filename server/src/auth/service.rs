//! Session validation and the authenticated-user extractor.

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chrono::{DateTime, Utc};
use oscc_core::{DEFAULT_SESSION_POLICY, SessionState, session_state};
use oscc_models::Role;
use sqlx::PgPool;

use crate::AppState;
use crate::auth::token;
use crate::error::AppError;

/// The authenticated caller, available to any handler that names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser {
    /// Database id of the account.
    pub user_id: i64,
    /// Login name.
    pub username: String,
    /// Human-readable name for the top bar.
    pub display_name: String,
    /// Role for permission checks and audit entries.
    pub role: Role,
    /// Storage hash of the session token (used by logout to revoke it).
    pub(crate) token_hash: String,
}

/// Validates a raw bearer token against a session row and touches it.
///
/// # Errors
///
/// Returns [`AppError::Unauthorized`] for unknown, revoked, or expired
/// sessions, and [`AppError::Internal`] when the stored role is unknown or
/// the database fails.
pub async fn authenticate(pool: &PgPool, raw_token: &str) -> Result<AuthUser, AppError> {
    let token_hash = token::token_hash(raw_token);

    let row = sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            String,
            DateTime<Utc>,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
        ),
    >(
        "SELECT u.id, u.username, u.display_name, u.role, \
                s.created_at, s.last_seen_at, s.revoked_at \
         FROM sessions s JOIN users u ON u.id = s.user_id \
         WHERE s.token_hash = $1 AND u.active",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?;

    let Some((user_id, username, display_name, role_raw, created_at, last_seen_at, revoked_at)) =
        row
    else {
        return Err(AppError::Unauthorized("invalid session"));
    };

    let role = Role::parse(&role_raw).ok_or(AppError::Internal)?;
    let now = Utc::now();
    if session_state(
        now,
        created_at,
        last_seen_at,
        revoked_at,
        DEFAULT_SESSION_POLICY,
    ) != SessionState::Active
    {
        return Err(AppError::Unauthorized("session expired"));
    }

    sqlx::query("UPDATE sessions SET last_seen_at = now() WHERE token_hash = $1")
        .bind(&token_hash)
        .execute(pool)
        .await?;

    Ok(AuthUser {
        user_id,
        username,
        display_name,
        role,
        token_hash,
    })
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized("missing bearer token"))?;
        let raw_token = header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized("missing bearer token"))?;
        authenticate(state.pool()?, raw_token).await
    }
}

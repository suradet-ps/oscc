//! `/api/v1/auth` routes: login, logout, and the current user.

use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use oscc_core::DEFAULT_SESSION_POLICY;
use oscc_models::{AuditAction, Role};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::service::AuthUser;
use super::{password, token};
use crate::AppState;
use crate::audit::{self, AuditEvent};
use crate::error::AppError;

/// Builds the auth router (mounted under `/api/v1/auth`).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: SecretString,
}

#[derive(Serialize)]
struct UserInfo {
    username: String,
    display_name: String,
    role: Role,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    expires_at: DateTime<Utc>,
    user: UserInfo,
}

/// Exchanges credentials for an opaque session token.
///
/// Unknown, inactive, and wrong-password accounts all answer the same
/// `401` with the same message, and all burn the same Argon2 cost.
async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let pool = state.pool()?;
    let attempted = request.username.trim().to_string();
    if attempted.is_empty() || request.password.expose_secret().is_empty() {
        return Err(AppError::BadRequest("username and password are required"));
    }

    let record = sqlx::query_as::<_, (i64, String, String, String, String, bool)>(
        "SELECT id, username, display_name, role, password_hash, active \
         FROM users WHERE username = $1",
    )
    .bind(&attempted)
    .fetch_optional(pool)
    .await?;

    let Some((user_id, stored_username, display_name, role_raw, password_hash, active)) = record
    else {
        password::dummy_verify(request.password.expose_secret());
        audit_login_failed(pool, &attempted).await?;
        return Err(AppError::Unauthorized("invalid credentials"));
    };

    if !active || !password::verify_password(request.password.expose_secret(), &password_hash) {
        audit_login_failed(pool, &stored_username).await?;
        return Err(AppError::Unauthorized("invalid credentials"));
    }

    let role = Role::parse(&role_raw).ok_or(AppError::Internal)?;
    let (raw_token, token_hash) = token::generate();
    let expires_at = Utc::now() + DEFAULT_SESSION_POLICY.absolute;

    sqlx::query("INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(pool)
        .await?;

    audit::append(
        pool,
        &AuditEvent {
            action: AuditAction::LoginSucceeded,
            actor: &stored_username,
            actor_role: Some(role),
            case_id: None,
            reason: None,
            detail: serde_json::json!({}),
        },
    )
    .await?;

    Ok(Json(LoginResponse {
        token: raw_token,
        expires_at,
        user: UserInfo {
            username: stored_username,
            display_name,
            role,
        },
    }))
}

/// Revokes the caller's session.
async fn logout(State(state): State<AppState>, user: AuthUser) -> Result<StatusCode, AppError> {
    let pool = state.pool()?;
    sqlx::query(
        "UPDATE sessions SET revoked_at = now() WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(&user.token_hash)
    .execute(pool)
    .await?;

    audit::append(
        pool,
        &AuditEvent {
            action: AuditAction::Logout,
            actor: &user.username,
            actor_role: Some(user.role),
            case_id: None,
            reason: None,
            detail: serde_json::json!({}),
        },
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Returns the current caller — the client's session check.
async fn me(user: AuthUser) -> Json<UserInfo> {
    Json(UserInfo {
        username: user.username,
        display_name: user.display_name,
        role: user.role,
    })
}

/// Writes the failed-attempt entry; the role is unknown by definition.
async fn audit_login_failed(pool: &PgPool, attempted: &str) -> Result<(), AppError> {
    audit::append(
        pool,
        &AuditEvent {
            action: AuditAction::LoginFailed,
            actor: attempted,
            actor_role: None,
            case_id: None,
            reason: None,
            detail: serde_json::json!({}),
        },
    )
    .await?;
    Ok(())
}

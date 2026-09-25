//! API error type.
//!
//! Bodies carry a machine-readable code and an English message; Thai
//! user-facing text lives in the client (`app/`, AGENTS.md §10). Internal
//! causes are logged PII-free and never echoed.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Every API failure maps to one of these.
#[derive(Debug)]
pub enum AppError {
    /// Missing or invalid credentials/session.
    Unauthorized(&'static str),
    /// Authenticated but not permitted.
    Forbidden,
    /// A dependency (usually the database) is not available.
    Unavailable(&'static str),
    /// The request itself is malformed.
    BadRequest(&'static str),
    /// Something failed and saying more would leak internals.
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: &'static str,
}

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, &'static str) {
        match self {
            Self::Unauthorized(message) => (StatusCode::UNAUTHORIZED, "unauthorized", message),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "insufficient permission",
            ),
            Self::Unavailable(message) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable", message),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "internal error",
            ),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error, message) = self.parts();
        (status, Json(ErrorBody { error, message })).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "database error");
        Self::Internal
    }
}

//! HTTP client for the OSCC API.
//!
//! The client never holds database credentials: the only secret it carries
//! is the opaque session token, kept in memory for the life of the session.

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

/// Where the API lives in development. Deployment sets the real LAN URL in
/// the login screen (persistence is a later milestone).
pub const DEFAULT_API_BASE: &str = "http://127.0.0.1:8080";

/// The signed-in user as the API reports them.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct UserInfo {
    /// Login name.
    pub username: String,
    /// Human-readable name for the top bar.
    pub display_name: String,
    /// Machine role name; the UI labels it.
    pub role: String,
}

/// A successful sign-in.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LoginResponse {
    /// Opaque bearer token (memory only).
    pub token: String,
    /// Absolute expiry, echoed by the server.
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Who signed in.
    pub user: UserInfo,
}

/// A failed API call, reduced to what the UI needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiError {
    /// HTTP status, or 0 when the request never reached the server.
    pub status: u16,
    /// Machine code from the response body, when present.
    pub code: String,
}

impl ApiError {
    /// The Thai message the user sees (UI strings live in `app/`).
    pub fn thai_message(&self) -> &'static str {
        match self.status {
            0 => "เชื่อมต่อเซิร์ฟเวอร์ไม่ได้",
            401 => "ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง",
            403 => "บัญชีนี้ไม่มีสิทธิ์ใช้งานส่วนนี้",
            503 => "เซิร์ฟเวอร์ยังไม่พร้อมใช้งานฐานข้อมูล",
            _ => match self.code.as_str() {
                "bad_request" => "กรอกข้อมูลไม่ครบถ้วน",
                _ => "เกิดข้อผิดพลาด กรุณาลองใหม่",
            },
        }
    }
}

fn network_error() -> ApiError {
    ApiError {
        status: 0,
        code: "network".to_string(),
    }
}

fn api_base(base: &str) -> String {
    base.trim_end_matches('/').to_string()
}

/// Signs in and returns the new session.
///
/// # Errors
///
/// Returns the API error; the caller renders [`ApiError::thai_message`].
pub async fn login(base: &str, username: &str, password: &str) -> Result<LoginResponse, ApiError> {
    #[derive(Serialize)]
    struct Body<'a> {
        username: &'a str,
        password: &'a str,
    }

    let response = Request::post(&format!("{}/api/v1/auth/login", api_base(base)))
        .json(&Body { username, password })
        .map_err(|_| network_error())?
        .send()
        .await
        .map_err(|_| network_error())?;

    if !response.ok() {
        return Err(ApiError {
            status: response.status(),
            code: parse_error_code(response).await,
        });
    }

    response
        .json::<LoginResponse>()
        .await
        .map_err(|_| ApiError {
            status: response.status(),
            code: "invalid_response".to_string(),
        })
}

/// Confirms the session and returns the current user.
///
/// # Errors
///
/// Returns the API error when the session is gone or the server is down.
pub async fn me(base: &str, token: &str) -> Result<UserInfo, ApiError> {
    let response = Request::get(&format!("{}/api/v1/auth/me", api_base(base)))
        .header("authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|_| network_error())?;

    if !response.ok() {
        return Err(ApiError {
            status: response.status(),
            code: parse_error_code(response).await,
        });
    }

    response.json::<UserInfo>().await.map_err(|_| ApiError {
        status: response.status(),
        code: "invalid_response".to_string(),
    })
}

/// Revokes the session server-side.
///
/// # Errors
///
/// Returns the API error when the call cannot be completed; the client
/// signs out locally either way.
pub async fn logout(base: &str, token: &str) -> Result<(), ApiError> {
    let response = Request::post(&format!("{}/api/v1/auth/logout", api_base(base)))
        .header("authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|_| network_error())?;

    if response.ok() {
        Ok(())
    } else {
        Err(ApiError {
            status: response.status(),
            code: parse_error_code(response).await,
        })
    }
}

/// Reads the `error` code from a failure body, defaulting when absent.
async fn parse_error_code(response: gloo_net::http::Response) -> String {
    #[derive(Deserialize)]
    struct Body {
        error: String,
    }

    response
        .json::<Body>()
        .await
        .map(|body| body.error)
        .unwrap_or_else(|_| "unknown".to_string())
}

//! HTTP client for the OSCC API.
//!
//! The client never holds database credentials: the only secret it carries
//! is the opaque session token, kept in memory for the life of the session.

use chrono::{DateTime, Utc};
use gloo_net::http::Request;
use serde::de::DeserializeOwned;
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
    pub expires_at: DateTime<Utc>,
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
            401 => "เซสชันหมดอายุ กรุณาเข้าสู่ระบบใหม่",
            403 => "บัญชีนี้ไม่มีสิทธิ์ใช้งานส่วนนี้",
            404 => "ไม่พบข้อมูลที่ค้นหา",
            503 => "เซิร์ฟเวอร์หรือ HOSxP ยังไม่พร้อมใช้งาน",
            _ => match self.code.as_str() {
                "bad_request" => "กรอกข้อมูลไม่ครบถ้วนหรือไม่ถูกต้อง",
                _ => "เกิดข้อผิดพลาด กรุณาลองใหม่",
            },
        }
    }
}

/// A queue row — identity-free by design.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CaseSummary {
    /// Non-identifying case id.
    pub case_id: String,
    /// Machine status name.
    pub status: String,
    /// Machine incident name.
    pub incident_type: String,
    /// Machine risk name.
    pub risk_level: String,
    /// Machine department name.
    pub owner_department: String,
    /// When the case was registered.
    pub reported_at: DateTime<Utc>,
}

/// Patient identity as the API masks it.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct MaskedPatient {
    /// Masked hospital number.
    pub hn: String,
    /// Masked national ID.
    pub cid: String,
    /// Masked name.
    pub name: String,
    /// When the intake snapshot was taken.
    pub snapshot_at: DateTime<Utc>,
}

/// One case with masked identity.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CaseDetail {
    /// Non-identifying case id.
    pub case_id: String,
    /// Machine status name.
    pub status: String,
    /// Machine incident name.
    pub incident_type: String,
    /// When the incident happened, if reported.
    pub incident_at: Option<DateTime<Utc>>,
    /// When the case was registered.
    pub reported_at: DateTime<Utc>,
    /// Machine risk name.
    pub risk_level: String,
    /// Machine department name.
    pub owner_department: String,
    /// Masked patient identity.
    pub patient: MaskedPatient,
}

/// Full identity, returned only by the audited reveal call.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RevealedPatient {
    /// Hospital number snapshot.
    pub hn: String,
    /// National ID snapshot.
    pub cid: String,
    /// Name snapshot.
    pub name: String,
    /// When the intake snapshot was taken.
    pub snapshot_at: DateTime<Utc>,
}

/// A masked lookup candidate.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PatientCandidate {
    /// Masked hospital number.
    pub hn: String,
    /// Masked national ID.
    pub cid: String,
    /// Masked name.
    pub name: String,
    /// Birth date as text, when HOSxP has one.
    pub birth_date: Option<String>,
    /// Sex as HOSxP records it.
    pub sex: Option<String>,
}

/// Intake payload.
#[derive(Clone, Debug, Serialize)]
pub struct CreateCaseInput {
    /// Hospital number, when known.
    pub hn: Option<String>,
    /// National ID, when known.
    pub cid: Option<String>,
    /// Machine incident name.
    pub incident_type: String,
    /// When the incident happened, if reported.
    pub incident_at: Option<DateTime<Utc>>,
    /// Machine risk name.
    pub risk_level: String,
    /// Machine department name.
    pub owner_department: String,
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

/// A signed-in API client: the base URL plus the session token.
#[derive(Clone, Debug)]
pub struct ApiClient {
    base: String,
    token: String,
}

impl ApiClient {
    /// Wraps a base URL and token.
    pub fn new(base: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            token: token.into(),
        }
    }

    /// Lists the case queue, newest first.
    ///
    /// # Errors
    ///
    /// Returns the API error.
    pub async fn list_cases(&self, status: Option<&str>) -> Result<Vec<CaseSummary>, ApiError> {
        let path = match status {
            Some(status) => format!("/api/v1/cases?status={status}"),
            None => "/api/v1/cases".to_string(),
        };
        self.get_json(&path).await
    }

    /// Loads one case with masked identity.
    ///
    /// # Errors
    ///
    /// Returns the API error.
    pub async fn get_case(&self, case_id: &str) -> Result<CaseDetail, ApiError> {
        self.get_json(&format!("/api/v1/cases/{case_id}")).await
    }

    /// Registers a case.
    ///
    /// # Errors
    ///
    /// Returns the API error.
    pub async fn create_case(&self, input: &CreateCaseInput) -> Result<CaseDetail, ApiError> {
        self.post_json("/api/v1/cases", input).await
    }

    /// Looks up a masked candidate by hospital number.
    ///
    /// # Errors
    ///
    /// Returns the API error.
    pub async fn lookup_hn(&self, hn: &str) -> Result<Vec<PatientCandidate>, ApiError> {
        self.get_json(&format!("/api/v1/patients/lookup?hn={hn}"))
            .await
    }

    /// Looks up a masked candidate by national ID.
    ///
    /// # Errors
    ///
    /// Returns the API error.
    pub async fn lookup_cid(&self, cid: &str) -> Result<Vec<PatientCandidate>, ApiError> {
        self.get_json(&format!("/api/v1/patients/lookup?cid={cid}"))
            .await
    }

    /// Reveals the full identity with a mandatory reason.
    ///
    /// # Errors
    ///
    /// Returns the API error.
    pub async fn reveal(&self, case_id: &str, reason: &str) -> Result<RevealedPatient, ApiError> {
        #[derive(Serialize)]
        struct Body<'a> {
            reason: &'a str,
        }
        self.post_json(&format!("/api/v1/cases/{case_id}/reveal"), &Body { reason })
            .await
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let response = Request::get(&format!("{}{path}", api_base(&self.base)))
            .header("authorization", &format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|_| network_error())?;

        if !response.ok() {
            return Err(ApiError {
                status: response.status(),
                code: parse_error_code(response).await,
            });
        }

        response.json::<T>().await.map_err(|_| ApiError {
            status: response.status(),
            code: "invalid_response".to_string(),
        })
    }

    async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let response = Request::post(&format!("{}{path}", api_base(&self.base)))
            .header("authorization", &format!("Bearer {}", self.token))
            .json(body)
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

        response.json::<T>().await.map_err(|_| ApiError {
            status: response.status(),
            code: "invalid_response".to_string(),
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

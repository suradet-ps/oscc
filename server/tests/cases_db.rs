//! Case registry flow against a disposable database (AGENTS.md §8): runs
//! only when `OSCC_TEST_DATABASE_URL` points at a test database. Without
//! the variable, the test reports success by skipping.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oscc_hosxp_connector::{PatientQuery, PatientRow};
use oscc_server::patients::{PatientSource, PatientSourceError, SharedPatientSource};
use oscc_server::{AppState, app};
use tower::ServiceExt;

/// A patient source with one canned patient, so the case flow needs no HOSxP.
struct FakeSource {
    rows: Vec<PatientRow>,
}

#[async_trait]
impl PatientSource for FakeSource {
    async fn find(&self, _query: PatientQuery) -> Result<Vec<PatientRow>, PatientSourceError> {
        Ok(self.rows.clone())
    }
}

fn fake_patient() -> PatientRow {
    PatientRow {
        hn: "12345".to_string(),
        cid: Some("1101701234567".to_string()),
        full_name_th: "สมชาย ทดสอบ".to_string(),
        birth_date: None,
        sex: Some("ชาย".to_string()),
    }
}

async fn json_of(
    response: axum::response::Response,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let body = response.into_body().collect().await?.to_bytes();
    Ok(serde_json::from_slice(&body)?)
}

#[tokio::test]
async fn intake_list_and_detail_round_trip_when_database_available()
-> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("OSCC_TEST_DATABASE_URL") else {
        println!("skipping: OSCC_TEST_DATABASE_URL is not set");
        return Ok(());
    };

    let pool = oscc_server::db::connect(&url).await?;
    oscc_server::db::migrate(&pool).await?;

    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let username = format!("test-nurse-{unique}");
    let password_hash = oscc_server::auth::password::hash_password("test-password")
        .map_err(|err| format!("hash failed: {err}"))?;
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, display_name, role, password_hash) \
         VALUES ($1, 'Test Nurse', 'er_nurse', $2) RETURNING id",
    )
    .bind(&username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await?;

    let patients: SharedPatientSource = Arc::new(FakeSource {
        rows: vec![fake_patient()],
    });
    let router = app(AppState::with_pool_and_patients(pool.clone(), patients));

    // Sign in for a bearer token.
    let login = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "username": username, "password": "test-password" })
                        .to_string(),
                ))?,
        )
        .await?;
    assert_eq!(login.status(), StatusCode::OK);
    let token = json_of(login).await?["token"]
        .as_str()
        .ok_or("token missing")?
        .to_string();

    // Intake.
    let create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cases")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "hn": "12345",
                        "incident_type": "domestic_violence",
                        "risk_level": "high",
                        "owner_department": "emergency"
                    })
                    .to_string(),
                ))?,
        )
        .await?;
    assert_eq!(create.status(), StatusCode::CREATED);
    let detail = json_of(create).await?;
    let case_id = detail["case_id"]
        .as_str()
        .ok_or("case_id missing")?
        .to_string();
    oscc_models::CaseId::parse(&case_id).map_err(|err| format!("bad case id: {err}"))?;
    assert_eq!(detail["status"], "intake");
    assert_eq!(detail["patient"]["hn"], "12****45");
    assert_eq!(detail["patient"]["cid"], "1-XXXX-XXXXX-XX-7");
    assert_eq!(detail["patient"]["name"], "สม*****");

    // Queue is identity-free.
    let list = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/cases")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(list.status(), StatusCode::OK);
    let queue = json_of(list).await?;
    let rows = queue.as_array().ok_or("queue is not an array")?;
    let ours = rows
        .iter()
        .find(|row| row["case_id"] == case_id)
        .ok_or("case missing from queue")?;
    assert!(
        ours.get("patient").is_none(),
        "queue rows must not carry identity"
    );

    // Detail.
    let get = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/cases/{case_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(get.status(), StatusCode::OK);
    let fetched = json_of(get).await?;
    assert_eq!(fetched["patient"]["name"], "สม*****");

    // Unknown case.
    let missing = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/cases/OSCC-2026-9999")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);

    // Both actions are on the audit chain.
    let actions: Vec<String> =
        sqlx::query_scalar("SELECT action FROM audit_entries WHERE case_id = $1 ORDER BY id")
            .bind(&case_id)
            .fetch_all(&pool)
            .await?;
    assert_eq!(actions, vec!["case_opened", "case_viewed"]);

    sqlx::query(
        "DELETE FROM patient_links WHERE case_pk = (SELECT id FROM cases WHERE case_id = $1)",
    )
    .bind(&case_id)
    .execute(&pool)
    .await?;
    sqlx::query("DELETE FROM cases WHERE case_id = $1")
        .bind(&case_id)
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await?;
    Ok(())
}

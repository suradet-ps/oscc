//! Database-backed auth flow (AGENTS.md §8): runs only when
//! `OSCC_TEST_DATABASE_URL` points at a disposable test database, never
//! production. Without the variable, the test reports success by skipping —
//! the default suite must stay DB-free.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oscc_server::{AppState, app};
use tower::ServiceExt;

#[tokio::test]
async fn login_me_logout_round_trip_when_database_available()
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
         VALUES ($1, $2, 'er_nurse', $3) RETURNING id",
    )
    .bind(&username)
    .bind("Test Nurse")
    .bind(&password_hash)
    .fetch_one(&pool)
    .await?;

    let router = app(AppState::with_pool(pool.clone()));

    let login_body =
        serde_json::json!({ "username": username, "password": "test-password" }).to_string();
    let login = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body))?,
        )
        .await?;
    assert_eq!(login.status(), StatusCode::OK);
    let body = login.into_body().collect().await?.to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body)?;
    let token = json["token"].as_str().ok_or("token missing")?.to_string();
    assert_eq!(json["user"]["role"], "er_nurse");

    let me = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(me.status(), StatusCode::OK);

    let logout = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);

    let after = router
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(after.status(), StatusCode::UNAUTHORIZED);

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

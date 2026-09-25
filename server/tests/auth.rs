//! Route-level auth tests that need no database (AGENTS-RUST.md §8.2).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use oscc_server::{AppState, app};
use tower::ServiceExt;

#[tokio::test]
async fn me_without_token_is_unauthorized() -> Result<(), Box<dyn std::error::Error>> {
    let response = app(AppState::new())
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn me_with_token_without_database_is_unavailable() -> Result<(), Box<dyn std::error::Error>> {
    let response = app(AppState::new())
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header("authorization", "Bearer deadbeef")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}

#[tokio::test]
async fn login_without_database_is_unavailable() -> Result<(), Box<dyn std::error::Error>> {
    let response = app(AppState::new())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "username": "nurse.a", "password": "x" }).to_string(),
                ))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}

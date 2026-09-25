//! API-level tests for the health endpoint (AGENTS-RUST.md §8.2).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oscc_server::{AppState, app};
use tower::ServiceExt;

#[tokio::test]
async fn healthz_returns_ok_json() -> Result<(), Box<dyn std::error::Error>> {
    let response = app(AppState::new())
        .oneshot(Request::builder().uri("/healthz").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await?.to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(json["status"], "ok");
    assert!(json["version"].is_string());
    assert!(json["uptime_secs"].is_u64());
    Ok(())
}

#[tokio::test]
async fn unknown_route_returns_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let response = app(AppState::new())
        .oneshot(Request::builder().uri("/nope").body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    Ok(())
}

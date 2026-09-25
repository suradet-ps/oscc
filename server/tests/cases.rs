//! Case route tests that need no database (AGENTS-RUST.md §8.2).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use oscc_server::{AppState, app};
use tower::ServiceExt;

#[tokio::test]
async fn cases_without_token_is_unauthorized() -> Result<(), Box<dyn std::error::Error>> {
    let response = app(AppState::new())
        .oneshot(
            Request::builder()
                .uri("/api/v1/cases")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn cases_with_token_without_database_is_unavailable() -> Result<(), Box<dyn std::error::Error>>
{
    let response = app(AppState::new())
        .oneshot(
            Request::builder()
                .uri("/api/v1/cases")
                .header("authorization", "Bearer deadbeef")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}

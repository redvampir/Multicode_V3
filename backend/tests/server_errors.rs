use axum::{body::Body, http::Request};
use axum::http::StatusCode;
use tower::ServiceExt;

use backend::config::ServerConfig;
use backend::server::{test_router, set_server_config, ErrorResponse};

fn setup() -> axum::Router {
    set_server_config(ServerConfig {
        host: "127.0.0.1".into(),
        port: 0,
        token: Some("secret".into()),
    });
    test_router()
}

#[tokio::test]
async fn parse_endpoint_unauthorized() {
    let app = setup();
    let body = serde_json::json!({ "content": "", "lang": "rust" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err.code, StatusCode::UNAUTHORIZED.as_u16());
    assert_eq!(err.message, "Unauthorized");
}

#[tokio::test]
async fn parse_endpoint_bad_request() {
    let app = setup();
    let body = serde_json::json!({ "content": "", "lang": "unknown" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/parse")
                .header("content-type", "application/json")
                .header("Authorization", "Bearer secret")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err.code, StatusCode::BAD_REQUEST.as_u16());
    assert_eq!(err.message, "Bad Request");
}

#[tokio::test]
async fn export_endpoint_unauthorized() {
    let app = setup();
    let body = serde_json::json!({ "content": "hello" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/export")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err.code, StatusCode::UNAUTHORIZED.as_u16());
    assert_eq!(err.message, "Unauthorized");
}

#[tokio::test]
async fn metadata_endpoint_unauthorized() {
    let app = setup();
    let body = serde_json::json!({
        "content": "",
        "lang": "rust",
        "meta": {"id": "1", "x": 0.0, "y": 0.0}
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/metadata")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let err: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err.code, StatusCode::UNAUTHORIZED.as_u16());
    assert_eq!(err.message, "Unauthorized");
}

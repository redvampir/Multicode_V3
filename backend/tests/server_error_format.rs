use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use backend::config::ServerConfig;
use backend::meta::{AiNote, VisualMeta};
use backend::server::{
    export_endpoint, metadata_endpoint, parse_endpoint, ErrorResponse, ExportRequest,
    MetadataRequest, ParseRequest, SERVER_CONFIG,
};
use std::collections::HashMap;

#[tokio::test]
async fn parse_endpoint_bad_request() {
    let _ = SERVER_CONFIG.set(ServerConfig {
        token: Some("secret".into()),
        ..Default::default()
    });
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        "Bearer secret".parse().unwrap(),
    );
    let req = ParseRequest {
        content: "test".into(),
        lang: "unknown".into(),
    };
    let (status, Json(err)) = parse_endpoint(headers, Json(req)).await.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        err,
        ErrorResponse {
            code: StatusCode::BAD_REQUEST.as_u16(),
            message: "Bad Request".into(),
        }
    );
}

#[tokio::test]
async fn export_endpoint_unauthorized() {
    let _ = SERVER_CONFIG.set(ServerConfig {
        token: Some("secret".into()),
        ..Default::default()
    });
    let headers = HeaderMap::new();
    let req = ExportRequest {
        content: "code".into(),
        strip_meta: false,
    };
    let (status, Json(err)) = export_endpoint(headers, Json(req)).await.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        err,
        ErrorResponse {
            code: StatusCode::UNAUTHORIZED.as_u16(),
            message: "Unauthorized".into(),
        }
    );
}

#[tokio::test]
async fn metadata_endpoint_unauthorized() {
    let _ = SERVER_CONFIG.set(ServerConfig {
        token: Some("secret".into()),
        ..Default::default()
    });
    let headers = HeaderMap::new();
    let req = MetadataRequest {
        content: String::new(),
        meta: VisualMeta {
            id: "1".into(),
            x: 0.0,
            y: 0.0,
            tags: vec![],
            origin: None,
            translations: HashMap::new(),
            ai: Some(AiNote::default()),
        },
        lang: "rust".into(),
    };
    let (status, Json(err)) = metadata_endpoint(headers, Json(req)).await.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        err,
        ErrorResponse {
            code: StatusCode::UNAUTHORIZED.as_u16(),
            message: "Unauthorized".into(),
        }
    );
}

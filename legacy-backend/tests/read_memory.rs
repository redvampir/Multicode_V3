use std::io::Write;

use axum::{extract::Query as AxumQuery, http::HeaderMap, Json};
use backend::config::ServerConfig;
use backend::server::{read_memory_endpoint, ReadMemoryQuery, SERVER_CONFIG};
use tempfile::NamedTempFile;
use tokio::fs;

fn init_config() {
    let _ = SERVER_CONFIG.set(ServerConfig {
        disable_auth: true,
        ..Default::default()
    });
}

#[tokio::test]
async fn reads_valid_range() {
    init_config();
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "Hello world").unwrap();
    file.flush().unwrap();
    let path = file.path().to_string_lossy().to_string();

    let response = read_memory_endpoint(
        HeaderMap::new(),
        AxumQuery(ReadMemoryQuery {
            file: path.clone(),
            range: Some("0:5".to_string()),
        }),
    )
    .await
    .unwrap();

    let Json(body) = response;
    assert_eq!(body.status, "ok");
    assert_eq!(body.chunk_start, 0);
    assert_eq!(body.chunk_end, 5);
    assert_eq!(body.content, "Hello");
    assert!(body.truncated);
    assert_eq!(body.encoding, "utf8");
}

#[tokio::test]
async fn trims_range_exceeding_size() {
    init_config();
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "Hello world").unwrap();
    file.flush().unwrap();
    let path = file.path().to_string_lossy().to_string();

    let response = read_memory_endpoint(
        HeaderMap::new(),
        AxumQuery(ReadMemoryQuery {
            file: path.clone(),
            range: Some("8:10".to_string()),
        }),
    )
    .await
    .unwrap();

    let Json(body) = response;
    assert_eq!(body.chunk_start, 8);
    assert_eq!(body.chunk_end, body.size);
    assert_eq!(body.content, "rld");
    assert!(!body.truncated);
}

#[tokio::test]
async fn fails_without_range() {
    init_config();
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_string_lossy().to_string();

    let error = read_memory_endpoint(
        HeaderMap::new(),
        AxumQuery(ReadMemoryQuery {
            file: path,
            range: None,
        }),
    )
    .await
    .err()
    .unwrap();

    assert_eq!(error.0, axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn reads_chunk_from_large_file() {
    init_config();
    let mut file = NamedTempFile::new().unwrap();
    let payload = "A".repeat(2 * 1024 * 1024);
    file.write_all(payload.as_bytes()).unwrap();
    file.flush().unwrap();
    let path = file.path().to_string_lossy().to_string();

    let response = read_memory_endpoint(
        HeaderMap::new(),
        AxumQuery(ReadMemoryQuery {
            file: path.clone(),
            range: Some("1024:2048".to_string()),
        }),
    )
    .await
    .unwrap();

    let Json(body) = response;
    assert_eq!(body.chunk_start, 1024);
    assert_eq!(body.chunk_end, 1024 + 2048);
    assert_eq!(body.content.len(), 2048);
    assert!(body.truncated);

    let size_on_disk = fs::metadata(path).await.unwrap().len();
    assert!(body.chunk_end < size_on_disk);
}

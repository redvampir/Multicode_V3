use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket}, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;
use serde::Deserialize;
use std::{env, net::SocketAddr};

use crate::{parse_blocks, upsert_meta, BlockInfo};
use crate::meta::{remove_all, VisualMeta};

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

fn auth(headers: &HeaderMap) -> bool {
    let token = env::var("API_TOKEN").unwrap_or_default();
    if token.is_empty() {
        return true;
    }
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|t| t == token)
        .unwrap_or(false)
}

#[derive(Deserialize)]
struct ParseRequest {
    content: String,
    lang: String,
}

async fn parse_endpoint(headers: HeaderMap, Json(req): Json<ParseRequest>) -> Result<Json<Vec<BlockInfo>>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    parse_blocks(req.content, req.lang)
        .map(Json)
        .ok_or(StatusCode::BAD_REQUEST)
}

#[derive(Deserialize)]
struct ExportRequest {
    content: String,
}

async fn export_endpoint(headers: HeaderMap, Json(req): Json<ExportRequest>) -> Result<Json<String>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(remove_all(&req.content)))
}

#[derive(Deserialize)]
struct MetadataRequest {
    content: String,
    meta: VisualMeta,
    lang: String,
}

async fn metadata_endpoint(headers: HeaderMap, Json(req): Json<MetadataRequest>) -> Result<Json<String>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(upsert_meta(req.content, req.meta, req.lang)))
}

async fn ws_handler(headers: HeaderMap, ws: WebSocketUpgrade, State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state.tx.clone())))
}

async fn handle_socket(socket: WebSocket, tx: broadcast::Sender<String>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();
    let tx_incoming = tx.clone();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                let _ = tx_incoming.send(text);
            }
        }
    });

    while let Ok(msg) = rx.recv().await {
        if sender.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

pub async fn run() {
    let (tx, _rx) = broadcast::channel::<String>(100);
    let state = AppState { tx };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/parse", post(parse_endpoint))
        .route("/export", post(export_endpoint))
        .route("/metadata", post(metadata_endpoint))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

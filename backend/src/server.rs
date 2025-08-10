use crate::config::ServerConfig;
use axum::extract::ws::Message;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::{SinkExt, StreamExt};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::{signal, sync::broadcast};
use tracing::{error, info};

use crate::meta::{remove_all, AiNote, VisualMeta};
use crate::{parse_blocks, upsert_meta, BlockInfo};

static SERVER_CONFIG: OnceCell<ServerConfig> = OnceCell::new();

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

fn auth(headers: &HeaderMap) -> bool {
    match SERVER_CONFIG.get().and_then(|c| c.token.as_deref()) {
        Some(token) if !token.is_empty() => headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|t| t == token)
            .unwrap_or(false),
        _ => true,
    }
}

#[derive(Deserialize)]
struct ParseRequest {
    content: String,
    lang: String,
}

async fn parse_endpoint(
    headers: HeaderMap,
    Json(req): Json<ParseRequest>,
) -> Result<Json<Vec<BlockInfo>>, StatusCode> {
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
    #[serde(default)]
    strip_meta: bool,
}

async fn export_endpoint(
    headers: HeaderMap,
    Json(req): Json<ExportRequest>,
) -> Result<Json<String>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let out = if req.strip_meta {
        remove_all(&req.content)
    } else {
        req.content
    };
    Ok(Json(out))
}

#[derive(Deserialize)]
struct MetadataRequest {
    content: String,
    meta: VisualMeta,
    lang: String,
}

async fn metadata_endpoint(
    headers: HeaderMap,
    Json(req): Json<MetadataRequest>,
) -> Result<Json<String>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(upsert_meta(req.content, req.meta, req.lang)))
}

#[derive(Deserialize)]
struct SuggestRequest {
    content: String,
    lang: String,
}

async fn suggest_endpoint(
    headers: HeaderMap,
    Json(_req): Json<SuggestRequest>,
) -> Result<Json<AiNote>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(AiNote {
        description: Some("Not implemented".into()),
        hints: Vec::new(),
    }))
}

async fn ws_handler(
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
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
    let cfg = ServerConfig::from_env();
    let _ = SERVER_CONFIG.set(cfg.clone());

    let (tx, _rx) = broadcast::channel::<String>(100);
    let state = AppState { tx };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/parse", post(parse_endpoint))
        .route("/export", post(export_endpoint))
        .route("/metadata", post(metadata_endpoint))
        .route("/suggest_ai_note", post(suggest_endpoint))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port)
        .parse()
        .expect("invalid address");
    info!("Listening on {}", addr);
    let ctrl_c = signal::ctrl_c();
    if let Err(e) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            if let Err(e) = ctrl_c.await {
                error!("failed to listen for shutdown signal: {e}");
            }
        })
        .await
    {
        error!("server error: {e}");
    }
}

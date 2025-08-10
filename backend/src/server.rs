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
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{signal, sync::broadcast, time};
use tracing::{error, info};

use crate::meta::{remove_all, AiNote, VisualMeta};
use crate::{parse_blocks, upsert_meta, BlockInfo};

static SERVER_CONFIG: OnceCell<ServerConfig> = OnceCell::new();

const MAX_CONNECTIONS: usize = 100;
const PING_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    connections: Arc<AtomicUsize>,
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

#[derive(Deserialize, Serialize)]
struct SuggestRequest {
    content: String,
    lang: String,
}

async fn suggest_endpoint(
    headers: HeaderMap,
    Json(req): Json<SuggestRequest>,
) -> Result<Json<AiNote>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let cfg = SERVER_CONFIG.get().cloned().unwrap_or_default();
    let client = reqwest::Client::new();
    let mut builder = client
        .post("https://api.example.com/suggest")
        .json(&req);

    if let Some(key) = cfg.api_key {
        builder = builder.header("Authorization", format!("Bearer {}", key));
    }

    let response = builder.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let note: AiNote = response
        .json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(note))
}

async fn ws_handler(
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let current = state.connections.fetch_add(1, Ordering::SeqCst);
    if current >= MAX_CONNECTIONS {
        state.connections.fetch_sub(1, Ordering::SeqCst);
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(ws.on_upgrade(move |socket| {
        handle_socket(socket, state.tx.clone(), state.connections.clone())
    }))
}

async fn handle_socket(
    socket: WebSocket,
    tx: broadcast::Sender<String>,
    connections: Arc<AtomicUsize>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();
    let tx_incoming = tx.clone();
    let awaiting_pong = Arc::new(AtomicBool::new(false));
    let pong_flag = awaiting_pong.clone();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let _ = tx_incoming.send(text);
                }
                Message::Pong(_) => {
                    pong_flag.store(false, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    });

    let mut interval = time::interval(PING_INTERVAL);
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if awaiting_pong.swap(true, Ordering::SeqCst) {
                    let _ = sender.send(Message::Close(None)).await;
                    break;
                }
                if sender.send(Message::Ping(Vec::new())).await.is_err() {
                    break;
                }
            }
            msg = rx.recv() => {
                match msg {
                    Ok(msg) => {
                        if sender.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }

    connections.fetch_sub(1, Ordering::SeqCst);
}

pub async fn run() {
    let cfg = ServerConfig::from_env();
    let _ = SERVER_CONFIG.set(cfg.clone());

    let (tx, _rx) = broadcast::channel::<String>(100);
    let state = AppState {
        tx,
        connections: Arc::new(AtomicUsize::new(0)),
    };
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

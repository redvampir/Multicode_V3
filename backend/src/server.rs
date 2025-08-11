use crate::config::ServerConfig;
use axum::extract::ws::Message;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, Query as AxumQuery, State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::{SinkExt, StreamExt};
use once_cell::sync::OnceCell;
use reqwest::Client;
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

use crate::meta::db;
use crate::meta::{remove_all, watch, AiNote, VisualMeta};
use crate::{
    blocks::{parse_blocks, upsert_meta},
    get_plugins_info, reload_plugins_state, set_plugin_enabled, BlockInfo, PluginInfo,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub static SERVER_CONFIG: OnceCell<ServerConfig> = OnceCell::new();

const MAX_CONNECTIONS: usize = 100;
const PING_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    connections: Arc<AtomicUsize>,
    db: SqlitePool,
}

#[cfg(test)]
pub fn test_state() -> AppState {
    use tokio::sync::broadcast;
    let (tx, _rx) = broadcast::channel(1);
    AppState {
        tx,
        connections: Arc::new(AtomicUsize::new(0)),
        db: SqlitePool::connect_lazy("sqlite::memory:").expect("mem db"),
    }
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
pub struct ParseRequest {
    content: String,
    lang: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ErrorResponse {
    code: u16,
    message: String,
}

pub async fn parse_endpoint(
    headers: HeaderMap,
    Json(req): Json<ParseRequest>,
) -> Result<Json<Vec<BlockInfo>>, (StatusCode, Json<ErrorResponse>)> {
    if !auth(&headers) {
        let status = StatusCode::UNAUTHORIZED;
        return Err((
            status,
            Json(ErrorResponse {
                code: status.as_u16(),
                message: "Unauthorized".into(),
            }),
        ));
    }
    parse_blocks(req.content, req.lang)
        .map(Json)
        .ok_or_else(|| {
            let status = StatusCode::BAD_REQUEST;
            (
                status,
                Json(ErrorResponse {
                    code: status.as_u16(),
                    message: "Bad Request".into(),
                }),
            )
        })
}

#[derive(Deserialize)]
pub struct ExportRequest {
    content: String,
    #[serde(default)]
    strip_meta: bool,
}

pub async fn export_endpoint(
    headers: HeaderMap,
    Json(req): Json<ExportRequest>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    if !auth(&headers) {
        let status = StatusCode::UNAUTHORIZED;
        return Err((
            status,
            Json(ErrorResponse {
                code: status.as_u16(),
                message: "Unauthorized".into(),
            }),
        ));
    }
    let out = if req.strip_meta {
        remove_all(&req.content)
    } else {
        req.content
    };
    Ok(Json(out))
}

#[derive(Deserialize)]
pub struct MetadataRequest {
    content: String,
    meta: VisualMeta,
    lang: String,
}

/// Insert or update metadata in the database.
pub async fn metadata_upsert_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<MetadataRequest>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    if !auth(&headers) {
        let status = StatusCode::UNAUTHORIZED;
        return Err((
            status,
            Json(ErrorResponse {
                code: status.as_u16(),
                message: "Unauthorized".into(),
            }),
        ));
    }
    if let Err(e) = db::upsert(&state.db, &req.meta).await {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        return Err((
            status,
            Json(ErrorResponse {
                code: status.as_u16(),
                message: format!("DB error: {e}"),
            }),
        ));
    }
    Ok(Json(upsert_meta(req.content, req.meta, req.lang)))
}

#[derive(Deserialize)]
struct MetaQuery {
    q: Option<String>,
}

/// List metadata entries filtered by query.
pub async fn metadata_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumQuery(params): AxumQuery<MetaQuery>,
) -> Result<Json<Vec<VisualMeta>>, (StatusCode, Json<ErrorResponse>)> {
    use crate::meta::query::{matches, parse};
    if !auth(&headers) {
        let status = StatusCode::UNAUTHORIZED;
        return Err((
            status,
            Json(ErrorResponse {
                code: status.as_u16(),
                message: "Unauthorized".into(),
            }),
        ));
    }
    let mut metas = match db::list(&state.db).await {
        Ok(m) => m,
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            return Err((
                status,
                Json(ErrorResponse {
                    code: status.as_u16(),
                    message: format!("DB error: {e}"),
                }),
            ));
        }
    };
    if let Some(q) = params.q.as_deref() {
        let expr = parse(q);
        metas = metas.into_iter().filter(|m| matches(m, &expr)).collect();
    }
    Ok(Json(metas))
}

pub async fn meta_history_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Vec<db::HistoryEntry>>, (StatusCode, Json<ErrorResponse>)> {
    if !auth(&headers) {
        let status = StatusCode::UNAUTHORIZED;
        return Err((
            status,
            Json(ErrorResponse {
                code: status.as_u16(),
                message: "Unauthorized".into(),
            }),
        ));
    }
    match db::history(&state.db, &id).await {
        Ok(hist) => Ok(Json(hist)),
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            Err((
                status,
                Json(ErrorResponse {
                    code: status.as_u16(),
                    message: format!("DB error: {e}"),
                }),
            ))
        }
    }
}

#[derive(Deserialize)]
pub struct RollbackRequest {
    timestamp: String,
}

pub async fn meta_rollback_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<RollbackRequest>,
) -> Result<Json<VisualMeta>, (StatusCode, Json<ErrorResponse>)> {
    if !auth(&headers) {
        let status = StatusCode::UNAUTHORIZED;
        return Err((
            status,
            Json(ErrorResponse {
                code: status.as_u16(),
                message: "Unauthorized".into(),
            }),
        ));
    }
    match db::rollback(&state.db, &id, &req.timestamp).await {
        Ok(meta) => Ok(Json(meta)),
        Err(sqlx::Error::RowNotFound) => {
            let status = StatusCode::NOT_FOUND;
            Err((
                status,
                Json(ErrorResponse {
                    code: status.as_u16(),
                    message: "History entry not found".into(),
                }),
            ))
        }
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            Err((
                status,
                Json(ErrorResponse {
                    code: status.as_u16(),
                    message: format!("DB error: {e}"),
                }),
            ))
        }
    }
}

async fn plugins_get(headers: HeaderMap) -> Result<Json<Vec<PluginInfo>>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(get_plugins_info()))
}

#[derive(Deserialize)]
struct PluginToggle {
    name: String,
    enabled: bool,
}

async fn plugins_update(
    headers: HeaderMap,
    Json(req): Json<PluginToggle>,
) -> Result<Json<Vec<PluginInfo>>, StatusCode> {
    if !auth(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    set_plugin_enabled(req.name, req.enabled).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(get_plugins_info()))
}

#[derive(Deserialize)]
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
    let cfg = SERVER_CONFIG
        .get()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let api_key = cfg
        .api_key
        .as_deref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = Client::new();
    let note = client
        .post("https://api.example.com/suggest")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .json::<AiNote>()
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

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = ServerConfig::from_env();
    let _ = SERVER_CONFIG.set(cfg.clone());

    reload_plugins_state();

    let (tx, _rx) = broadcast::channel::<String>(100);

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:visual_meta.db")
        .await
        .map_err(|e| {
            error!("connect db: {e}");
            e
        })?;
    db::init(&db).await.map_err(|e| {
        error!("init db: {e}");
        e
    })?;

    watch::spawn(tx.clone());
    let state = AppState {
        tx,
        connections: Arc::new(AtomicUsize::new(0)),
        db,
    };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/parse", post(parse_endpoint))
        .route("/export", post(export_endpoint))
        .route(
            "/metadata",
            get(metadata_endpoint).post(metadata_upsert_endpoint),
        )
        .route("/meta/:id/history", get(meta_history_endpoint))
        .route("/meta/:id/rollback", post(meta_rollback_endpoint))
        .route("/plugins", get(plugins_get).post(plugins_update))
        .route("/suggest_ai_note", post(suggest_endpoint))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse().map_err(|e| {
        error!("invalid address: {e}");
        e
    })?;
    info!("Listening on {}", addr);
    let ctrl_c = signal::ctrl_c();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            if let Err(e) = ctrl_c.await {
                error!("failed to listen for shutdown signal: {e}");
            }
        })
        .await
        .map_err(|e| {
            error!("server error: {e}");
            e
        })?;
    Ok(())
}

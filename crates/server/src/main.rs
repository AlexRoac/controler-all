use anyhow::Context;
use axum::{
    Json, Router, body::Body,
    extract::{ConnectInfo, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::SinkExt;
use qrcode::{QrCode, render::unicode};
use rand::Rng;
use remotepad_input_controller::InputController;
use remotepad_protocol::{ClientMessage, MAX_MESSAGE_BYTES, PROTOCOL_VERSION, PairRequest, PairResponse, SequenceGate, ServerInfo, ServerMessage};
use remotepad_virtual_gamepad::{DiagnosticGamepad, VirtualGamepadBackend};
use rust_embed::RustEmbed;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, net::{IpAddr, SocketAddr}, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(RustEmbed)]
#[folder = "../../apps/web/dist/"]
struct WebAssets;

#[derive(Clone)]
struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    name: String,
    addresses: Vec<IpAddr>,
    port: u16,
    pair_code: String,
    tokens: Mutex<HashMap<[u8; 32], Instant>>,
    pair_attempts: Mutex<HashMap<IpAddr, (Instant, u8)>>,
    active: AtomicBool,
    input: Mutex<Option<InputController>>,
    gamepad: Arc<dyn VirtualGamepadBackend>,
}

impl AppState {
    fn new(name: String, addresses: Vec<IpAddr>, port: u16, pair_code: String) -> Self {
        let input = match InputController::new() {
            Ok(controller) => Some(controller),
            Err(error) => { warn!(%error, "desktop input is unavailable"); None }
        };
        Self { inner: Arc::new(Inner { name, addresses, port, pair_code, tokens: Mutex::new(HashMap::new()), pair_attempts: Mutex::new(HashMap::new()), active: AtomicBool::new(false), input: Mutex::new(input), gamepad: DiagnosticGamepad::new() }) }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("remotepad_server=info".parse()?)).init();
    let port = std::env::var("REMOTEPAD_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(8787);
    let name = hostname::get().unwrap_or_default().to_string_lossy().to_string();
    let mut addresses: Vec<IpAddr> = local_ip_address::list_afinet_netifas().unwrap_or_default().into_iter().map(|(_, ip)| ip).filter(|ip| match ip { IpAddr::V4(v4) => v4.is_private() && !v4.is_loopback() && !v4.is_link_local(), IpAddr::V6(_) => false }).collect();
    addresses.sort(); addresses.dedup();
    let pair_code = format!("{:06}", rand::rng().random_range(0..1_000_000));
    let state = AppState::new(name.clone(), addresses.clone(), port, pair_code.clone());
    let cors = allowed_cors()?;
    let app = Router::new()
        .route("/api/info", get(info_handler))
        .route("/api/pair", post(pair_handler))
        .route("/ws", get(ws_handler))
        .fallback(get(static_handler))
        .layer(RequestBodyLimitLayer::new(MAX_MESSAGE_BYTES))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    println!("\nRemotePad — {name}");
    println!("Pairing code: {pair_code} (new code after each restart)");
    for ip in &addresses {
        let url = format!("http://{ip}:{port}");
        println!("\nOpen: {url}");
        if let Ok(code) = QrCode::new(url.as_bytes()) { println!("{}", code.render::<unicode::Dense1x2>().quiet_zone(false).build()); }
    }
    if addresses.is_empty() { println!("No LAN IPv4 detected. Local URL: http://127.0.0.1:{port}"); }
    println!("Press Ctrl+C to stop safely.\n");

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.context("could not bind RemotePad server")?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).with_graceful_shutdown(shutdown_signal()).await?;
    cleanup(&state).await;
    Ok(())
}

async fn static_handler(uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    let requested = if requested.is_empty() { "index.html" } else { requested };
    let (asset, served_path, immutable) = match WebAssets::get(requested) {
        Some(asset) => (asset, requested, requested.starts_with("assets/")),
        None => match WebAssets::get("index.html") {
            Some(index) => (index, "index.html", false),
            None => return (StatusCode::INTERNAL_SERVER_ERROR, "embedded web application is missing").into_response(),
        },
    };
    let mime = mime_guess::from_path(served_path).first_or_octet_stream();
    let cache = if immutable { "public, max-age=31536000, immutable" } else { "no-cache" };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::CACHE_CONTROL, cache)
        .body(Body::from(asset.data.into_owned()))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn allowed_cors() -> anyhow::Result<CorsLayer> {
    let mut origins = vec![HeaderValue::from_static("http://localhost:5173"), HeaderValue::from_static("http://127.0.0.1:5173")];
    if let Ok(extra) = std::env::var("REMOTEPAD_ALLOWED_ORIGINS") {
        for origin in extra.split(',').map(str::trim).filter(|s| !s.is_empty()) { origins.push(HeaderValue::from_str(origin)?); }
    }
    Ok(CorsLayer::new().allow_origin(origins).allow_methods([Method::GET, Method::POST]).allow_headers([axum::http::header::CONTENT_TYPE]))
}

async fn info_handler(State(state): State<AppState>) -> Json<ServerInfo> {
    Json(ServerInfo { name: state.inner.name.clone(), addresses: state.inner.addresses.iter().map(ToString::to_string).collect(), port: state.inner.port, connected: state.inner.active.load(Ordering::Acquire), gamepad_backend: state.inner.gamepad.name().into(), protocol_version: PROTOCOL_VERSION })
}

async fn pair_handler(State(state): State<AppState>, ConnectInfo(peer): ConnectInfo<SocketAddr>, Json(request): Json<PairRequest>) -> Response {
    {
        let mut attempts = state.inner.pair_attempts.lock().await;
        attempts.retain(|_, (started, _)| started.elapsed() < Duration::from_secs(60));
        let entry = attempts.entry(peer.ip()).or_insert((Instant::now(), 0));
        entry.1 = entry.1.saturating_add(1);
        if entry.1 > 6 { return (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({"error":"too many pairing attempts"}))).into_response(); }
    }
    if request.client_name.trim().is_empty() || request.client_name.len() > 80 || request.code != state.inner.pair_code {
        warn!(ip = %peer.ip(), "rejected pairing attempt");
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"invalid pairing code"}))).into_response();
    }
    let token = format!("{}{}", Uuid::new_v4(), Uuid::new_v4());
    state.inner.tokens.lock().await.insert(hash_token(&token), Instant::now() + Duration::from_secs(12 * 60 * 60));
    info!(client = %request.client_name, ip = %peer.ip(), "paired client");
    Json(PairResponse { token, expires_in_seconds: 43_200 }).into_response()
}

async fn ws_handler(State(state): State<AppState>, headers: HeaderMap, ws: WebSocketUpgrade, ConnectInfo(peer): ConnectInfo<SocketAddr>) -> Response {
    if !origin_allowed(&state, &headers, peer.ip()) { return StatusCode::FORBIDDEN.into_response(); }
    ws.max_message_size(MAX_MESSAGE_BYTES).max_frame_size(MAX_MESSAGE_BYTES).on_upgrade(move |socket| client_session(socket, state, peer))
}

fn origin_allowed(state: &AppState, headers: &HeaderMap, peer_ip: IpAddr) -> bool {
    let Some(origin) = headers.get(axum::http::header::ORIGIN).and_then(|value| value.to_str().ok()) else { return peer_ip.is_loopback(); };
    let mut allowed = vec![format!("http://localhost:{}", state.inner.port), format!("http://127.0.0.1:{}", state.inner.port), "http://localhost:5173".into(), "http://127.0.0.1:5173".into()];
    allowed.extend(state.inner.addresses.iter().map(|ip| format!("http://{ip}:{}", state.inner.port)));
    if let Ok(extra) = std::env::var("REMOTEPAD_ALLOWED_ORIGINS") { allowed.extend(extra.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned)); }
    allowed.iter().any(|candidate| candidate == origin)
}

async fn client_session(mut socket: WebSocket, state: AppState, peer: SocketAddr) {
    let authenticated = match tokio::time::timeout(Duration::from_secs(5), socket.recv()).await {
        Ok(Some(Ok(Message::Text(text)))) => authenticate(&state, &text).await,
        _ => false,
    };
    if !authenticated {
        let _ = send_json(&mut socket, &ServerMessage::Error { code: "unauthorized".into(), message: "Pair again with the current code".into() }).await;
        let _ = socket.close().await;
        return;
    }
    if state.inner.active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
        let _ = send_json(&mut socket, &ServerMessage::Error { code: "controller_busy".into(), message: "Another controller is active".into() }).await;
        let _ = socket.close().await;
        return;
    }
    let session_id = Uuid::new_v4().to_string();
    info!(%peer, %session_id, "controller connected");
    if send_json(&mut socket, &ServerMessage::Authenticated { session_id: session_id.clone(), server_name: state.inner.name.clone() }).await.is_err() { cleanup(&state).await; return; }
    let mut sequences = SequenceGate::default();
    let mut window = Instant::now();
    let mut messages_in_window = 0u32;
    loop {
        let next = tokio::time::timeout(Duration::from_secs(12), socket.recv()).await;
        let message = match next { Ok(Some(Ok(message))) => message, Ok(Some(Err(error))) => { warn!(%error, "websocket error"); break; }, _ => break };
        if window.elapsed() >= Duration::from_secs(1) { window = Instant::now(); messages_in_window = 0; }
        messages_in_window += 1;
        if messages_in_window > 240 { warn!(%peer, "rate limit exceeded"); break; }
        let Message::Text(text) = message else { if matches!(message, Message::Close(_)) { break; } continue; };
        if text.len() > MAX_MESSAGE_BYTES { break; }
        let parsed = match serde_json::from_str::<ClientMessage>(&text) {
            Ok(message) => message,
            Err(error) => { let _ = send_json(&mut socket, &ServerMessage::Error { code: "invalid_message".into(), message: error.to_string() }).await; continue; }
        };
        if let Some(sequence) = parsed.sequence() && !sequences.accept(sequence) { continue; }
        if process_message(&state, &mut socket, parsed).await.is_err() { break; }
    }
    cleanup(&state).await;
    info!(%peer, %session_id, "controller disconnected; inputs released");
}

async fn authenticate(state: &AppState, raw: &str) -> bool {
    let Ok(ClientMessage::Authenticate { token, protocol_version }) = serde_json::from_str::<ClientMessage>(raw) else { return false; };
    if protocol_version != PROTOCOL_VERSION { return false; }
    let key = hash_token(&token);
    let mut tokens = state.inner.tokens.lock().await;
    tokens.retain(|_, expires| *expires > Instant::now());
    tokens.get(&key).is_some()
}

async fn process_message(state: &AppState, socket: &mut WebSocket, message: ClientMessage) -> anyhow::Result<()> {
    match message {
        ClientMessage::Heartbeat { client_time } => send_json(socket, &ServerMessage::HeartbeatAck { client_time, server_time: unix_millis() }).await?,
        ClientMessage::GamepadState { state: gamepad } => {
            if !gamepad.is_valid() { anyhow::bail!("invalid gamepad values"); }
            state.inner.gamepad.update(&gamepad).await?;
            send_json(socket, &ServerMessage::Diagnostic { sequence: gamepad.sequence, gamepad, backend: state.inner.gamepad.name().into() }).await?;
        }
        ClientMessage::MouseMove { dx, dy, .. } => with_input(state, |i| i.move_mouse(dx, dy)).await?,
        ClientMessage::MouseButton { button, pressed, .. } => with_input(state, |i| i.mouse_button(button, pressed)).await?,
        ClientMessage::MouseScroll { delta, .. } => with_input(state, |i| i.scroll(delta)).await?,
        ClientMessage::TextInput { text, .. } => with_input(state, |i| i.text(&text)).await?,
        ClientMessage::Key { key, pressed, modifiers, .. } => with_input(state, |i| i.key(key, pressed, &modifiers)).await?,
        ClientMessage::Media { action, .. } => with_input(state, |i| i.media(action)).await?,
        ClientMessage::EmergencyStop { .. } => { cleanup(state).await; anyhow::bail!("emergency stop"); }
        ClientMessage::Authenticate { .. } => {}
    }
    Ok(())
}

async fn with_input<F>(state: &AppState, operation: F) -> anyhow::Result<()> where F: FnOnce(&mut InputController) -> Result<(), remotepad_input_controller::InputError> {
    let mut guard = state.inner.input.lock().await;
    let controller = guard.as_mut().context("desktop input backend unavailable")?;
    operation(controller)?; Ok(())
}

async fn cleanup(state: &AppState) {
    state.inner.gamepad.reset().await.ok();
    if let Some(input) = state.inner.input.lock().await.as_mut() { input.release_all(); }
    state.inner.active.store(false, Ordering::Release);
}

async fn send_json(socket: &mut WebSocket, message: &ServerMessage) -> anyhow::Result<()> {
    socket.send(Message::Text(serde_json::to_string(message)?.into())).await?; Ok(())
}
fn hash_token(token: &str) -> [u8; 32] { Sha256::digest(token.as_bytes()).into() }
fn unix_millis() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64 }
async fn shutdown_signal() { let _ = tokio::signal::ctrl_c().await; info!("shutdown requested"); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokens_are_not_stored_in_plain_text_form() { assert_ne!(hash_token("secret").as_slice(), b"secret"); }
    #[test]
    fn malformed_message_is_rejected() { assert!(serde_json::from_str::<ClientMessage>("not json").is_err()); }
}

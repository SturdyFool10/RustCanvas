#![allow(unused_imports)]
use appstate::{AppState, ConnectionId, MessageSender};
use axum::Router;

use axum::extract::Path;
use axum::extract::ws::{Message, WebSocketUpgrade};
use axum::response::Html;
use axum::routing::{get, post};
use axum_extra::response::*;
use futures::{Future, SinkExt, StreamExt};
use prost::Message as _;
use prost_reflect::bytes::Bytes;
use protocol::detect_message_type;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::*;

pub async fn start_webserver(state: AppState) {
    start_listening(state).await;
}

fn get_router(state: AppState) -> axum::Router {
    Router::new()
        .route("/", get(|| async { get_index() }))
        .route("/index.js", get(|| async { get_index_js() }))
        .route("/jquery.min.js", get(|| async { get_jquery() }))
        .route("/proto-client.js", get(|| async { get_proto_js() }))
        .route("/stylesheet.css", get(|| async { get_stylesheet() }))
        .route(
            "/ws",
            get(
                |ws: WebSocketUpgrade, state: axum::extract::State<AppState>| {
                    handle_ws_upgrade(ws, state)
                },
            ),
        )
        .with_state(state)
}

async fn start_listening(state: AppState) {
    let router = get_router(state.clone());
    let (internal, external) = parse_config(state).await;
    info!("Starting webserver on {} ({})", &external, &internal);
    let listener = TcpListener::bind(&internal)
        .await
        .expect("Failed to bind to address");
    let server = axum::serve(listener, router).await;
    if let Err(e) = server {
        error!("Failed to start web server: \n\t{}", e);
    }
}

//returns the functional and display strings for the network and interface
async fn parse_config(state: AppState) -> (String, String) {
    let config = state.config.lock().await;
    let network = config.network.clone();
    let interface = network.interface.clone();
    let port = network.port;
    drop(network);
    drop(config);
    let functional = format!("{}:{}", interface, port);
    let display_interface: String = match interface.as_str() {
        "0.0.0.0" => "*".to_string(),
        "127.0.0.1" => "localhost".to_string(),
        _ => interface.clone(),
    };
    let display: String = format!("{}:{}", display_interface, port);
    (functional, display)
}

// NOTE TO SELF: This handles the HTTP->WS upgrade dance
// Remember: ws.on_upgrade needs an async block inside!
async fn handle_ws_upgrade(
    ws: WebSocketUpgrade,
    state: axum::extract::State<AppState>,
) -> axum::response::Response {
    let state = state.0.clone();
    ws.on_upgrade(move |socket| async move {
        // Handle client in this async block, which will be spawned by axum
        handle_client(socket, state.clone()).await;
    })
}

// Main entry point for WebSockets - this gets called for each connection
// TODO: Add metrics tracking here later?
async fn handle_client(socket: axum::extract::ws::WebSocket, state: AppState) {
    debug!("New WebSocket connection established");

    // Set up the connection and register it with the app state
    let connection_id = setup_connection(socket, state.clone()).await;

    // Once the connection is terminated, clean it up
    state.ws_connections.unregister(connection_id).await;
    debug!("WebSocket connection {} closed", connection_id);
}

// Split the connection into the parts we need and set everything up
// This was tricky to get right - don't mess with the order of operations
async fn setup_connection(socket: axum::extract::ws::WebSocket, state: AppState) -> ConnectionId {
    // Split the socket into sender and receiver
    let (sender, receiver) = socket.split();

    // Set up the message plumbing and get this connection registered
    let (connection_id, rx) = register_connection(state.clone()).await;
    info!("Registered new WebSocket connection: {}", connection_id);

    // Spin up the worker tasks - each one does a specific job
    let tasks = spawn_connection_tasks(sender, receiver, rx, state, connection_id);

    // Wait until something breaks, then clean everything up
    // Could add reconnect logic here later if needed
    wait_for_tasks_completion(tasks).await;

    // Return the connection ID for cleanup
    connection_id
}

// Create a channel and register the connection with our global state
// IMPORTANT: This is how clients get their unique IDs
async fn register_connection(state: AppState) -> (ConnectionId, mpsc::Receiver<Message>) {
    // Channel for sending messages from various tasks to the WebSocket
    let (tx, rx) = mpsc::channel::<Message>(100);

    // Make a sender and register it - this lets other parts of the app message this client
    let message_sender = MessageSender::new(tx);
    let connection_id = state.ws_connections.register(message_sender).await;

    (connection_id, rx)
}

// Fire up the three tasks we need for each connection
// Got tired of copy-pasting this everywhere, so made it a function
fn spawn_connection_tasks(
    sender: futures::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
    receiver: futures::stream::SplitStream<axum::extract::ws::WebSocket>,
    rx: mpsc::Receiver<Message>,
    state: AppState,
    conn_id: ConnectionId,
) -> (
    tokio::task::JoinHandle<()>,
    tokio::task::JoinHandle<()>,
    tokio::task::JoinHandle<()>,
) {
    let send_task = spawn_send_task(sender, rx, conn_id);
    let heartbeat_task = spawn_heartbeat_task(state.clone(), conn_id);
    let receive_task = spawn_receive_task(receiver, state, conn_id);

    (send_task, heartbeat_task, receive_task)
}

// Wait for any task to finish, then kill them all
// This prevents resource leaks - learned this the hard way...
async fn wait_for_tasks_completion(
    (mut send_task, mut heartbeat_task, mut receive_task): (
        tokio::task::JoinHandle<()>,
        tokio::task::JoinHandle<()>,
        tokio::task::JoinHandle<()>,
    ),
) {
    tokio::select! {
        _ = &mut send_task => {},
        _ = &mut heartbeat_task => {},
        _ = &mut receive_task => {},
    }

    // Abort all tasks when one completes/fails
    send_task.abort();
    heartbeat_task.abort();
    receive_task.abort();
}

// Task 1: Send messages from our app to the client
// Pretty straightforward - just a loop that pulls from channel & sends to socket
fn spawn_send_task(
    sender: futures::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
    rx: mpsc::Receiver<Message>,
    conn_id: ConnectionId,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        process_outgoing_messages(sender, rx, conn_id).await;
    })
}

/// Spawns a task that sends periodic pings to keep the connection alive
fn spawn_heartbeat_task(state: AppState, conn_id: ConnectionId) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        send_heartbeats(state, conn_id).await;
    })
}

/// Spawns a task that processes incoming messages from the WebSocket
fn spawn_receive_task(
    receiver: futures::stream::SplitStream<axum::extract::ws::WebSocket>,
    state: AppState,
    conn_id: ConnectionId,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        process_incoming_messages(receiver, state, conn_id).await;
    })
}

/// Process outgoing messages from the channel to the WebSocket
async fn process_outgoing_messages(
    mut sender: futures::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
    mut rx: mpsc::Receiver<Message>,
    conn_id: ConnectionId,
) {
    while let Some(message) = rx.recv().await {
        if let Err(e) = sender.send(message).await {
            error!(
                "Connection {}: Error sending WebSocket message: {}",
                conn_id, e
            );
            break;
        }
    }
    debug!("Send task for connection {} terminated", conn_id);
}

// Keep the connection alive with pings
// 30 sec interval seems to work well with most clients & proxies
async fn send_heartbeats(state: AppState, conn_id: ConnectionId) {
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        // Bail out if app is shutting down
        if !state.running.load(std::sync::atomic::Ordering::Relaxed) {
            debug!("Heartbeat task shutting down");
            break;
        }

        // Only ping if client still exists (avoid zombies)
        if let Some(sender) = state.ws_connections.get(conn_id).await {
            if sender
                .send(Message::Ping(Bytes::from(vec![])))
                .await
                .is_err()
            {
                break;
            }
        } else {
            break;
        }
    }

    debug!("Heartbeat task for connection {} terminated", conn_id);
}

// Process stuff coming from the client
// Just basic handling for now - actual message processing happens elsewhere
async fn process_incoming_messages(
    mut receiver: futures::stream::SplitStream<axum::extract::ws::WebSocket>,
    state: AppState,
    conn_id: ConnectionId,
) {
    let mut last_pong = Instant::now();
    let timeout = Duration::from_secs(90); // 3x the ping interval seems to work well

    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                // No message handling here - that's for the application layer
                trace!(
                    "Connection {}: Received text message of length {}",
                    conn_id,
                    text.len()
                );
            }
            Ok(Message::Binary(data)) => {
                // Binary messages just get logged - actual handling elsewhere
                trace!(
                    "Connection {}: Received binary data of size: {} bytes: \n\t{:02X?}",
                    conn_id,
                    data.len(),
                    &data
                );
                // --- Type detection debug ---
                // Use the descriptor set embedded at compile time
                let descriptor_bytes = include_bytes!("../../protocol/src/descriptor_set.bin");
                if let Some(message_type) = protocol::get_proto_type(&data, descriptor_bytes) {
                    debug!(
                        "Connection {}: Detected protobuf message type: {}",
                        conn_id, message_type
                    );
                } else {
                    debug!(
                        "Connection {}: Failed to detect protobuf message type: No message type could decode the provided blob",
                        conn_id
                    );
                }
                // --- End type detection debug ---
            }
            Ok(Message::Close(_)) => {
                debug!("Connection {}: Client initiated close", conn_id);
                break;
            }
            Ok(Message::Ping(data)) => {
                // Gotta respond to pings - WS protocol requirement
                if let Some(sender) = state.ws_connections.get(conn_id).await {
                    if sender.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
            }
            Ok(Message::Pong(_)) => {
                // Client is still alive, reset the deadman switch
                last_pong = Instant::now();
                // Silently update timestamp, no logging needed
            }
            Err(e) => {
                debug!("Connection {}: WebSocket error: {}", conn_id, e);
                break;
            }
        }

        // Check if client ghosted us
        if last_pong.elapsed() > timeout {
            debug!("Connection {}: Client timed out", conn_id);
            break;
        }
    }

    debug!("Receive task for connection {} terminated", conn_id);
}

fn get_index() -> Html<String> {
    include_str!("htmlsrc/index.html").to_string().into()
}

fn get_index_js() -> JavaScript<String> {
    include_str!("htmlsrc/index.js").to_string().into()
}

fn get_jquery() -> JavaScript<String> {
    include_str!("htmlsrc/jquery.min.js").to_string().into()
}

fn get_stylesheet() -> Css<String> {
    include_str!("htmlsrc/stylesheet.css").to_string().into()
}

fn get_proto_js() -> JavaScript<String> {
    include_str!("htmlsrc/proto-client.js").to_string().into()
}

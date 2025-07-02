mod websocket;

use axum::extract::ws::Message;
use config::Config;
use db::DatabaseConnection;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
pub use websocket::{BinaryMessage, ConnectionId, ConnectionRegistry, MessageSender, TextMessage};

// Implement trait for axum WebSocket Message
impl TextMessage for Message {
    fn create_text_message(text: String) -> Self {
        Message::Text(text.into())
    }
}

impl BinaryMessage for Message {
    fn create_binary_message(data: Vec<u8>) -> Self {
        Message::Binary(axum::body::Bytes::from(data))
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<Config>>,
    pub db: Arc<Mutex<DatabaseConnection>>,
    pub running: Arc<AtomicBool>,
    pub ws_connections: ConnectionRegistry<Message>,
}
impl AppState {
    pub fn new(config: Config, db: DatabaseConnection) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            db: Arc::new(Mutex::new(db)),
            running: Arc::new(AtomicBool::new(true)),
            ws_connections: ConnectionRegistry::new(),
        }
    }
}

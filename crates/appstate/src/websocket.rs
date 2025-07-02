// Dependencies we need for the connection system
// HashMap: track connections, Arc/Mutex: thread safety, mpsc: message channels
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc};

// Simple ID type for clients - just a wrapper around a counter
// Using a newtype pattern here to avoid mixing up with other u64s
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ConnectionId(pub u64);

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Message sender for talking to a specific client
// Generic over message type so we can use different WS implementations
#[derive(Clone)]
pub struct MessageSender<T> {
    tx: mpsc::Sender<T>,
}

impl<T> MessageSender<T>
where
    T: Clone + Send + 'static,
{
    pub fn new(tx: mpsc::Sender<T>) -> Self {
        Self { tx }
    }

    // Basic send function - just passes through to the channel
    // Returns error if the client disconnected
    pub async fn send(&self, msg: T) -> Result<(), mpsc::error::SendError<T>> {
        self.tx.send(msg).await
    }
}

// Trait to abstract text message creation
// Needed because different WS implementations have different message types
pub trait TextMessage {
    fn create_text_message(text: String) -> Self;
}

// Same idea but for binary data
// This lets us avoid hardcoding to axum's message format
pub trait BinaryMessage {
    fn create_binary_message(data: Vec<u8>) -> Self;
}

// Add text sending capabilities if the message type supports it
// This is conditional - only available if T implements TextMessage
impl<T> MessageSender<T>
where
    T: TextMessage + Send + 'static,
{
    // Convenience wrapper for sending text - makes the API nicer
    pub async fn send_text(
        &self,
        text: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<T>> {
        self.tx.send(T::create_text_message(text.into())).await
    }
}

// Same pattern for binary - again, only available if T has binary capabilities
impl<T> MessageSender<T>
where
    T: BinaryMessage + Send + 'static,
{
    // Send raw bytes to the client
    pub async fn send_binary(
        &self,
        data: impl Into<Vec<u8>>,
    ) -> Result<(), mpsc::error::SendError<T>> {
        self.tx.send(T::create_binary_message(data.into())).await
    }
}

// The core connection manager - tracks all active clients
// Using RwLock for better concurrency (many reads, few writes)
#[derive(Clone)]
pub struct ConnectionRegistry<T> {
    connections: Arc<RwLock<HashMap<ConnectionId, MessageSender<T>>>>,
    next_id: Arc<Mutex<u64>>, // Counter for generating unique IDs
}

impl<T> ConnectionRegistry<T>
where
    T: Clone + Send + 'static,
{
    // Create fresh registry - start with empty map
    // Starting IDs at 1 because 0 feels like a sentinel value
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)), // Start IDs from 1
        }
    }

    // Add a new connection to the system
    // Returns its unique ID that can be used to message it later
    pub async fn register(&self, sender: MessageSender<T>) -> ConnectionId {
        let mut id_guard = self.next_id.lock().await;
        let id = ConnectionId(*id_guard);
        *id_guard += 1; // Increment for next time

        let mut connections = self.connections.write().await;
        connections.insert(id, sender);
        id
    }

    // Clean up when a client disconnects
    // Returns true if we actually removed something
    pub async fn unregister(&self, id: ConnectionId) -> bool {
        let mut connections = self.connections.write().await;
        connections.remove(&id).is_some()
    }

    // Look up a client by ID
    // Returns None if it doesn't exist/disconnected
    pub async fn get(&self, id: ConnectionId) -> Option<MessageSender<T>> {
        let connections = self.connections.read().await;
        connections.get(&id).cloned()
    }

    // Send the same message to all connected clients
    // Failures are ignored - common pattern for broadcast
    pub async fn broadcast(&self, msg: T) {
        let connections = self.connections.read().await;
        for sender in connections.values() {
            // Don't care about errors here - it's fine if some clients miss a broadcast
            let _ = sender.send(msg.clone()).await;
        }
    }

    // How many clients are currently connected?
    // Useful for debugging and stats
    pub async fn count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    // Get IDs of all connected clients
    // Useful for iterating through connections when needed
    pub async fn all_ids(&self) -> Vec<ConnectionId> {
        let connections = self.connections.read().await;
        connections.keys().copied().collect()
    }
}

// Add text broadcasting if message type supports it
// Same conditional pattern as with MessageSender
impl<T> ConnectionRegistry<T>
where
    T: TextMessage + Clone + Send + 'static,
{
    // Simpler API for broadcasting text
    // This is used a lot, so worth having a dedicated method
    pub async fn broadcast_text(&self, text: impl Into<String> + Clone) {
        let text = text.into();
        let connections = self.connections.read().await;
        for sender in connections.values() {
            // Again, don't care about errors in broadcast scenarios
            let _ = sender.send_text(text.clone()).await;
        }
    }
}

// And the same for binary broadcasts
// Not used as often but good to have for completeness
impl<T> ConnectionRegistry<T>
where
    T: BinaryMessage + Clone + Send + 'static,
{
    // Send raw bytes to all clients
    pub async fn broadcast_binary(&self, data: impl Into<Vec<u8>> + Clone) {
        let data = data.into();
        let connections = self.connections.read().await;
        for sender in connections.values() {
            // Ignore send errors as usual for broadcasts
            let _ = sender.send_binary(data.clone()).await;
        }
    }
}

// Implement Default so we can use this with struct field defaults
// Just delegates to new() to avoid duplicating logic
impl<T> Default for ConnectionRegistry<T>
where
    T: Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

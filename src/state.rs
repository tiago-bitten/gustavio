use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub username: String,
    pub ip: String,
    pub tcp_port: u16,
    pub last_seen: std::time::Instant,
}

/// Wraps an OwnedWriteHalf so it can be shared (stored in state + used by readers for acks).
pub type SharedWriter = Arc<Mutex<OwnedWriteHalf>>;

pub struct SharedState {
    /// Discovered peers (peer_id -> info)
    pub peers: Mutex<HashMap<String, PeerInfo>>,
    /// Active TCP write halves (peer_id -> writer)
    pub connections: Mutex<HashMap<String, SharedWriter>>,
}

impl SharedState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            peers: Mutex::new(HashMap::new()),
            connections: Mutex::new(HashMap::new()),
        })
    }
}

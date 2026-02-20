use serde::{Deserialize, Serialize};

// ── UDP Discovery ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UdpPacket {
    #[serde(rename = "announce")]
    Announce {
        peer_id: String,
        username: String,
        tcp_port: u16,
    },
    #[serde(rename = "goodbye")]
    Goodbye { peer_id: String },
}

// ── TCP Messages ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TcpMessage {
    Hello {
        peer_id: String,
        username: String,
    },
    DirectMessage {
        id: String,
        from_id: String,
        from_name: String,
        content: String,
        timestamp: String,
    },
    GroupMessage {
        id: String,
        group_id: String,
        from_id: String,
        from_name: String,
        content: String,
        timestamp: String,
    },
    Ack {
        message_id: String,
        status: String,
    },
    GroupCreate {
        group_id: String,
        name: String,
        creator_id: String,
        members: Vec<String>,
    },
    GroupMemberAdd {
        group_id: String,
        peer_id: String,
    },
    GroupMemberRemove {
        group_id: String,
        peer_id: String,
    },
}

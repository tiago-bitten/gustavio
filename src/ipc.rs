use serde::{Deserialize, Serialize};

/// Commands coming from the JS frontend via `window.ipc.postMessage(json)`.
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd")]
pub enum IpcCommand {
    #[serde(rename = "set_username")]
    SetUsername { username: String },
    #[serde(rename = "send_message")]
    SendMessage { peer_id: String, content: String },
    #[serde(rename = "send_group_message")]
    SendGroupMessage { group_id: String, content: String },
    #[serde(rename = "load_history")]
    LoadHistory { conversation_id: String },
    #[serde(rename = "create_group")]
    CreateGroup { name: String, members: Vec<String> },
    #[serde(rename = "get_peers")]
    GetPeers,
    #[serde(rename = "get_groups")]
    GetGroups,
    #[serde(rename = "mark_read")]
    MarkRead { conversation_id: String },
}

/// Builds JS eval strings for sending data from Rust into the WebView.
pub fn js_call(event: &str, data: &impl Serialize) -> String {
    let json = serde_json::to_string(data).unwrap_or_else(|_| "null".into());
    format!(
        "if(window.onRustMessage){{window.onRustMessage({},{})}};",
        serde_json::to_string(event).unwrap(),
        json
    )
}

use serde::{Deserialize, Serialize};

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
    #[serde(rename = "set_always_on_top")]
    SetAlwaysOnTop { enabled: bool },
}

pub fn js_call(event: &str, data: &impl Serialize) -> String {
    let json = serde_json::to_string(data).unwrap_or_else(|_| "null".into());
    format!(
        "if(window.onRustMessage){{window.onRustMessage({},{})}};",
        serde_json::to_string(event).unwrap(),
        json
    )
}

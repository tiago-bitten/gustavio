use crate::app_event::AppEvent;
use crate::db::{Database, MessageRow};
use crate::discovery;
use crate::ipc::{js_call, IpcCommand};
use crate::network;
use crate::protocol::TcpMessage;
use crate::state::SharedState;

use std::sync::Arc;
use tao::event_loop::EventLoopProxy;
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;

const TCP_PORT: u16 = 9999;

/// Spawns the tokio runtime on a background thread.
/// Returns an mpsc::Sender for IPC commands from the WebView.
pub fn start(proxy: EventLoopProxy<AppEvent>) -> mpsc::UnboundedSender<String> {
    let (tx, rx) = mpsc::unbounded_channel::<String>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async move {
            run(rx, proxy).await;
        });
    });

    tx
}

async fn run(mut rx: mpsc::UnboundedReceiver<String>, proxy: EventLoopProxy<AppEvent>) {
    // Open database
    let db = Database::open().expect("Failed to open database");
    let db = Arc::new(TokioMutex::new(db));

    // Load or create peer_id
    let (peer_id, username) = {
        let d = db.lock().await;
        let peer_id = match d.get_config("peer_id") {
            Some(id) => id,
            None => {
                let id = uuid::Uuid::new_v4().to_string();
                d.set_config("peer_id", &id).unwrap();
                id
            }
        };
        let username = d.get_config("username");
        (peer_id, username)
    };

    // Send config to UI
    #[derive(serde::Serialize)]
    struct ConfigInfo {
        peer_id: String,
        username: Option<String>,
    }
    let js = js_call(
        "config_loaded",
        &ConfigInfo {
            peer_id: peer_id.clone(),
            username: username.clone(),
        },
    );
    let _ = proxy.send_event(AppEvent::EvalScript(js));

    // Create shared state
    let state = SharedState::new();

    // If we already have a username, start networking
    let networking_started = Arc::new(TokioMutex::new(username.is_some()));
    if username.is_some() {
        start_networking(
            peer_id.clone(),
            username.clone().unwrap(),
            state.clone(),
            db.clone(),
            proxy.clone(),
        )
        .await;
        // Also send groups
        let d = db.lock().await;
        let groups = d.get_groups();
        drop(d);
        let js = js_call("group_list", &groups);
        let _ = proxy.send_event(AppEvent::EvalScript(js));
    }

    // Process IPC commands
    while let Some(raw) = rx.recv().await {
        let cmd: IpcCommand = match serde_json::from_str(&raw) {
            Ok(c) => c,
            Err(e) => {
                let js = js_call("error", &format!("Invalid command: {e}"));
                let _ = proxy.send_event(AppEvent::EvalScript(js));
                continue;
            }
        };

        match cmd {
            IpcCommand::SetUsername { username } => {
                {
                    let d = db.lock().await;
                    d.set_config("username", &username).unwrap();
                }
                // Start networking if not started
                let mut started = networking_started.lock().await;
                if !*started {
                    start_networking(
                        peer_id.clone(),
                        username.clone(),
                        state.clone(),
                        db.clone(),
                        proxy.clone(),
                    )
                    .await;
                    *started = true;
                }
                #[derive(serde::Serialize)]
                struct ConfigInfo {
                    peer_id: String,
                    username: Option<String>,
                }
                let js = js_call(
                    "config_loaded",
                    &ConfigInfo {
                        peer_id: peer_id.clone(),
                        username: Some(username),
                    },
                );
                let _ = proxy.send_event(AppEvent::EvalScript(js));
            }
            IpcCommand::SendMessage { peer_id: target_id, content } => {
                let timestamp = chrono::Utc::now().to_rfc3339();
                let msg_id = uuid::Uuid::new_v4().to_string();

                // Get our username
                let uname = {
                    let d = db.lock().await;
                    d.get_config("username").unwrap_or_default()
                };

                // Save to DB
                let row = MessageRow {
                    id: msg_id.clone(),
                    conversation_id: target_id.clone(),
                    from_id: peer_id.clone(),
                    from_name: uname.clone(),
                    content: content.clone(),
                    timestamp: timestamp.clone(),
                    is_group: false,
                    status: "sent".into(),
                };
                {
                    let d = db.lock().await;
                    let _ = d.insert_message(&row);
                }

                // Ensure connection exists
                {
                    let conns = state.connections.lock().await;
                    if !conns.contains_key(&target_id) {
                        drop(conns);
                        // Try to connect
                        let peers = state.peers.lock().await;
                        if let Some(peer_info) = peers.get(&target_id) {
                            let ip = peer_info.ip.clone();
                            let port = peer_info.tcp_port;
                            drop(peers);
                            network::connect_to_peer(
                                &ip,
                                port,
                                &peer_id,
                                &uname,
                                state.clone(),
                                db.clone(),
                                proxy.clone(),
                            )
                            .await;
                        }
                    }
                }

                // Send TCP message
                let tcp_msg = TcpMessage::DirectMessage {
                    id: msg_id,
                    from_id: peer_id.clone(),
                    from_name: uname,
                    content,
                    timestamp,
                };
                if let Err(e) = network::send_to_peer(&target_id, &tcp_msg, &state).await {
                    let js = js_call("error", &format!("Send failed: {e}"));
                    let _ = proxy.send_event(AppEvent::EvalScript(js));
                }

                // Push our own message to UI
                let js = js_call("incoming_message", &row);
                let _ = proxy.send_event(AppEvent::EvalScript(js));
            }
            IpcCommand::SendGroupMessage { group_id, content } => {
                let timestamp = chrono::Utc::now().to_rfc3339();
                let msg_id = uuid::Uuid::new_v4().to_string();

                let uname = {
                    let d = db.lock().await;
                    d.get_config("username").unwrap_or_default()
                };

                // Save to DB
                let row = MessageRow {
                    id: msg_id.clone(),
                    conversation_id: group_id.clone(),
                    from_id: peer_id.clone(),
                    from_name: uname.clone(),
                    content: content.clone(),
                    timestamp: timestamp.clone(),
                    is_group: true,
                    status: "sent".into(),
                };
                {
                    let d = db.lock().await;
                    let _ = d.insert_message(&row);
                }

                // Get group members
                let members = {
                    let d = db.lock().await;
                    d.get_group_members(&group_id)
                };

                // Fan-out to all members
                let tcp_msg = TcpMessage::GroupMessage {
                    id: msg_id,
                    group_id: group_id.clone(),
                    from_id: peer_id.clone(),
                    from_name: uname.clone(),
                    content,
                    timestamp,
                };
                for member_id in &members {
                    if member_id == &peer_id {
                        continue;
                    }
                    // Ensure connection
                    {
                        let conns = state.connections.lock().await;
                        if !conns.contains_key(member_id) {
                            drop(conns);
                            let peers = state.peers.lock().await;
                            if let Some(pi) = peers.get(member_id) {
                                let ip = pi.ip.clone();
                                let port = pi.tcp_port;
                                drop(peers);
                                network::connect_to_peer(
                                    &ip,
                                    port,
                                    &peer_id,
                                    &uname,
                                    state.clone(),
                                    db.clone(),
                                    proxy.clone(),
                                )
                                .await;
                            }
                        }
                    }
                    let _ = network::send_to_peer(member_id, &tcp_msg, &state).await;
                }

                // Push own message to UI
                let js = js_call("incoming_message", &row);
                let _ = proxy.send_event(AppEvent::EvalScript(js));
            }
            IpcCommand::LoadHistory { conversation_id } => {
                let messages = {
                    let d = db.lock().await;
                    d.load_history(&conversation_id, 200)
                };
                let js = js_call("history", &messages);
                let _ = proxy.send_event(AppEvent::EvalScript(js));
            }
            IpcCommand::CreateGroup { name, members } => {
                let group_id = uuid::Uuid::new_v4().to_string();
                {
                    let d = db.lock().await;
                    let _ = d.create_group(&group_id, &name, &peer_id);
                    // Add ourselves
                    let _ = d.add_group_member(&group_id, &peer_id);
                    for m in &members {
                        let _ = d.add_group_member(&group_id, m);
                    }
                }

                let uname = {
                    let d = db.lock().await;
                    d.get_config("username").unwrap_or_default()
                };

                // Notify all members via TCP
                let mut all_members = members.clone();
                all_members.push(peer_id.clone());
                let tcp_msg = TcpMessage::GroupCreate {
                    group_id: group_id.clone(),
                    name: name.clone(),
                    creator_id: peer_id.clone(),
                    members: all_members,
                };
                for member_id in &members {
                    // Ensure connection
                    {
                        let conns = state.connections.lock().await;
                        if !conns.contains_key(member_id) {
                            drop(conns);
                            let peers = state.peers.lock().await;
                            if let Some(pi) = peers.get(member_id) {
                                let ip = pi.ip.clone();
                                let port = pi.tcp_port;
                                drop(peers);
                                network::connect_to_peer(
                                    &ip,
                                    port,
                                    &peer_id,
                                    &uname,
                                    state.clone(),
                                    db.clone(),
                                    proxy.clone(),
                                )
                                .await;
                            }
                        }
                    }
                    let _ = network::send_to_peer(member_id, &tcp_msg, &state).await;
                }

                // Refresh groups in UI
                let d = db.lock().await;
                let groups = d.get_groups();
                drop(d);
                let js = js_call("group_list", &groups);
                let _ = proxy.send_event(AppEvent::EvalScript(js));

                let js = js_call("group_created", &group_id);
                let _ = proxy.send_event(AppEvent::EvalScript(js));
            }
            IpcCommand::GetPeers => {
                discovery::send_peer_list(&state, &proxy).await;
            }
            IpcCommand::GetGroups => {
                let d = db.lock().await;
                let groups = d.get_groups();
                drop(d);
                let js = js_call("group_list", &groups);
                let _ = proxy.send_event(AppEvent::EvalScript(js));
            }
            IpcCommand::MarkRead { conversation_id: _ } => {
                // Could update unread counts in DB — for now, handled client-side
            }
        }
    }
}

async fn start_networking(
    peer_id: String,
    username: String,
    state: Arc<SharedState>,
    db: Arc<TokioMutex<Database>>,
    proxy: EventLoopProxy<AppEvent>,
) {
    // Start TCP listener
    let pid = peer_id.clone();
    let uname = username.clone();
    let st = state.clone();
    let d = db.clone();
    let px = proxy.clone();
    tokio::spawn(async move {
        network::run_listener(pid, uname, st, d, px).await;
    });

    // Start UDP discovery
    let pid = peer_id.clone();
    let uname = username.clone();
    let st = state.clone();
    let px = proxy.clone();
    tokio::spawn(async move {
        discovery::run(pid, uname, TCP_PORT, st, px).await;
    });
}

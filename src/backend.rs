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

/// Try to connect to a peer if not already connected.
async fn ensure_connected(
    target_peer_id: &str,
    my_peer_id: &str,
    my_username: &str,
    state: &Arc<SharedState>,
    db: &Arc<TokioMutex<Database>>,
    proxy: &EventLoopProxy<AppEvent>,
) {
    {
        let conns = state.connections.lock().await;
        if conns.contains_key(target_peer_id) {
            return;
        }
    }

    let (ip, port) = {
        let peers = state.peers.lock().await;
        match peers.get(target_peer_id) {
            Some(p) => (p.ip.clone(), p.tcp_port),
            None => return,
        }
    };

    if let Err(e) = network::connect_to_peer(
        &ip,
        port,
        my_peer_id,
        my_username,
        state.clone(),
        db.clone(),
        proxy.clone(),
    )
    .await
    {
        eprintln!("Connect to peer failed: {e}");
    }
}

/// Send message to peer with retry: if first send fails, drop dead connection,
/// reconnect, and try once more.
async fn send_with_retry(
    target_peer_id: &str,
    msg: &TcpMessage,
    my_peer_id: &str,
    my_username: &str,
    state: &Arc<SharedState>,
    db: &Arc<TokioMutex<Database>>,
    proxy: &EventLoopProxy<AppEvent>,
) -> Result<(), String> {
    // First attempt
    ensure_connected(target_peer_id, my_peer_id, my_username, state, db, proxy).await;
    match network::send_to_peer(target_peer_id, msg, state).await {
        Ok(()) => return Ok(()),
        Err(e) => {
            eprintln!("Send failed (will retry): {e}");
        }
    }

    // Remove dead connection and retry
    network::remove_connection(target_peer_id, state).await;
    ensure_connected(target_peer_id, my_peer_id, my_username, state, db, proxy).await;
    network::send_to_peer(target_peer_id, msg, state).await
}

async fn run(mut rx: mpsc::UnboundedReceiver<String>, proxy: EventLoopProxy<AppEvent>) {
    let db = Database::open().expect("Failed to open database");
    let db = Arc::new(TokioMutex::new(db));

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

    let state = SharedState::new();

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
        let d = db.lock().await;
        let groups = d.get_groups();
        drop(d);
        let js = js_call("group_list", &groups);
        let _ = proxy.send_event(AppEvent::EvalScript(js));
    }

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
                let js = js_call(
                    "config_loaded",
                    &ConfigInfo {
                        peer_id: peer_id.clone(),
                        username: Some(username),
                    },
                );
                let _ = proxy.send_event(AppEvent::EvalScript(js));
            }

            IpcCommand::SendMessage {
                peer_id: target_id,
                content,
            } => {
                let timestamp = chrono::Utc::now().to_rfc3339();
                let msg_id = uuid::Uuid::new_v4().to_string();

                let uname = {
                    let d = db.lock().await;
                    d.get_config("username").unwrap_or_default()
                };

                let tcp_msg = TcpMessage::DirectMessage {
                    id: msg_id.clone(),
                    from_id: peer_id.clone(),
                    from_name: uname.clone(),
                    content: content.clone(),
                    timestamp: timestamp.clone(),
                };

                let status = match send_with_retry(
                    &target_id, &tcp_msg, &peer_id, &uname, &state, &db, &proxy,
                )
                .await
                {
                    Ok(_) => "sent",
                    Err(e) => {
                        let js = js_call("error", &format!("Envio falhou: {e}"));
                        let _ = proxy.send_event(AppEvent::EvalScript(js));
                        "failed"
                    }
                };

                let row = MessageRow {
                    id: msg_id,
                    conversation_id: target_id,
                    from_id: peer_id.clone(),
                    from_name: uname,
                    content,
                    timestamp,
                    is_group: false,
                    status: status.into(),
                };
                {
                    let d = db.lock().await;
                    let _ = d.insert_message(&row);
                }
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

                let members = {
                    let d = db.lock().await;
                    d.get_group_members(&group_id)
                };

                let tcp_msg = TcpMessage::GroupMessage {
                    id: msg_id.clone(),
                    group_id: group_id.clone(),
                    from_id: peer_id.clone(),
                    from_name: uname.clone(),
                    content: content.clone(),
                    timestamp: timestamp.clone(),
                };

                for member_id in &members {
                    if member_id == &peer_id {
                        continue;
                    }
                    if let Err(e) = send_with_retry(
                        member_id, &tcp_msg, &peer_id, &uname, &state, &db, &proxy,
                    )
                    .await
                    {
                        eprintln!("Group send to {member_id}: {e}");
                    }
                }

                let row = MessageRow {
                    id: msg_id,
                    conversation_id: group_id,
                    from_id: peer_id.clone(),
                    from_name: uname,
                    content,
                    timestamp,
                    is_group: true,
                    status: "sent".into(),
                };
                {
                    let d = db.lock().await;
                    let _ = d.insert_message(&row);
                }
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
                    let _ = d.add_group_member(&group_id, &peer_id);
                    for m in &members {
                        let _ = d.add_group_member(&group_id, m);
                    }
                }

                let uname = {
                    let d = db.lock().await;
                    d.get_config("username").unwrap_or_default()
                };

                let mut all_members = members.clone();
                all_members.push(peer_id.clone());
                let tcp_msg = TcpMessage::GroupCreate {
                    group_id: group_id.clone(),
                    name: name.clone(),
                    creator_id: peer_id.clone(),
                    members: all_members,
                };
                for member_id in &members {
                    let _ = send_with_retry(
                        member_id, &tcp_msg, &peer_id, &uname, &state, &db, &proxy,
                    )
                    .await;
                }

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

            IpcCommand::MarkRead { .. } => {}

            IpcCommand::SetAlwaysOnTop { enabled } => {
                let _ = proxy.send_event(AppEvent::SetAlwaysOnTop(enabled));
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
    let pid = peer_id.clone();
    let uname = username.clone();
    let st = state.clone();
    let d = db.clone();
    let px = proxy.clone();
    tokio::spawn(async move {
        network::run_listener(pid, uname, st, d, px).await;
    });

    let pid = peer_id.clone();
    let uname = username.clone();
    let st = state.clone();
    let px = proxy.clone();
    tokio::spawn(async move {
        discovery::run(pid, uname, TCP_PORT, st, px).await;
    });
}

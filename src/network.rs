use crate::app_event::AppEvent;
use crate::db::{Database, MessageRow};
use crate::ipc::js_call;
use crate::protocol::TcpMessage;
use crate::state::SharedState;

use std::sync::Arc;
use tao::event_loop::EventLoopProxy;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex as TokioMutex;

const TCP_PORT: u16 = 9999;

/// Start the TCP listener that accepts connections from peers.
pub async fn run_listener(
    my_peer_id: String,
    my_username: String,
    state: Arc<SharedState>,
    db: Arc<TokioMutex<Database>>,
    proxy: EventLoopProxy<AppEvent>,
) {
    let addr = format!("0.0.0.0:{TCP_PORT}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("TCP listen error: {e}");
            return;
        }
    };

    loop {
        let (stream, _addr) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };
        let pid = my_peer_id.clone();
        let uname = my_username.clone();
        let st = state.clone();
        let d = db.clone();
        let px = proxy.clone();
        tokio::spawn(async move {
            handle_connection(stream, pid, uname, st, d, px).await;
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    my_peer_id: String,
    my_username: String,
    state: Arc<SharedState>,
    db: Arc<TokioMutex<Database>>,
    proxy: EventLoopProxy<AppEvent>,
) {
    let (rd, wr) = stream.into_split();
    let mut reader = BufReader::new(rd);

    // Send our Hello
    let hello = TcpMessage::Hello {
        peer_id: my_peer_id.clone(),
        username: my_username.clone(),
    };
    let mut hello_json = serde_json::to_string(&hello).unwrap();
    hello_json.push('\n');

    let wr = Arc::new(tokio::sync::Mutex::new(wr));
    {
        let mut w = wr.lock().await;
        if w.write_all(hello_json.as_bytes()).await.is_err() {
            return;
        }
    }

    let mut remote_peer_id = String::new();
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let Ok(msg) = serde_json::from_str::<TcpMessage>(trimmed) else {
                    continue;
                };
                match msg {
                    TcpMessage::Hello {
                        peer_id,
                        username: _,
                    } => {
                        remote_peer_id = peer_id.clone();
                        // Store the write half for sending messages later
                        // We clone the Arc, but we need to move the actual writer
                        // This is handled by registering in connections
                    }
                    TcpMessage::DirectMessage {
                        ref id,
                        ref from_id,
                        ref from_name,
                        ref content,
                        ref timestamp,
                    } => {
                        // Save to DB
                        let row = MessageRow {
                            id: id.clone(),
                            conversation_id: from_id.clone(),
                            from_id: from_id.clone(),
                            from_name: from_name.clone(),
                            content: content.clone(),
                            timestamp: timestamp.clone(),
                            is_group: false,
                            status: "delivered".into(),
                        };
                        {
                            let d = db.lock().await;
                            let _ = d.insert_message(&row);
                        }
                        // Push to UI
                        let js = js_call("incoming_message", &row);
                        let _ = proxy.send_event(AppEvent::EvalScript(js));
                        // Send ack
                        let ack = TcpMessage::Ack {
                            message_id: id.clone(),
                            status: "delivered".into(),
                        };
                        let mut ack_json = serde_json::to_string(&ack).unwrap();
                        ack_json.push('\n');
                        let mut w = wr.lock().await;
                        let _ = w.write_all(ack_json.as_bytes()).await;
                    }
                    TcpMessage::GroupMessage {
                        ref id,
                        ref group_id,
                        ref from_id,
                        ref from_name,
                        ref content,
                        ref timestamp,
                    } => {
                        let row = MessageRow {
                            id: id.clone(),
                            conversation_id: group_id.clone(),
                            from_id: from_id.clone(),
                            from_name: from_name.clone(),
                            content: content.clone(),
                            timestamp: timestamp.clone(),
                            is_group: true,
                            status: "delivered".into(),
                        };
                        {
                            let d = db.lock().await;
                            let _ = d.insert_message(&row);
                        }
                        let js = js_call("incoming_message", &row);
                        let _ = proxy.send_event(AppEvent::EvalScript(js));
                    }
                    TcpMessage::Ack {
                        ref message_id,
                        ref status,
                    } => {
                        {
                            let d = db.lock().await;
                            let _ = d.update_message_status(message_id, status);
                        }
                        #[derive(serde::Serialize)]
                        struct AckInfo {
                            message_id: String,
                            status: String,
                        }
                        let js = js_call(
                            "message_ack",
                            &AckInfo {
                                message_id: message_id.clone(),
                                status: status.clone(),
                            },
                        );
                        let _ = proxy.send_event(AppEvent::EvalScript(js));
                    }
                    TcpMessage::GroupCreate {
                        ref group_id,
                        ref name,
                        ref creator_id,
                        ref members,
                    } => {
                        let d = db.lock().await;
                        let _ = d.create_group(group_id, name, creator_id);
                        for m in members {
                            let _ = d.add_group_member(group_id, m);
                        }
                        drop(d);
                        // Refresh groups in UI
                        let d = db.lock().await;
                        let groups = d.get_groups();
                        drop(d);
                        let js = js_call("group_list", &groups);
                        let _ = proxy.send_event(AppEvent::EvalScript(js));
                    }
                    TcpMessage::GroupMemberAdd {
                        ref group_id,
                        ref peer_id,
                    } => {
                        let d = db.lock().await;
                        let _ = d.add_group_member(group_id, peer_id);
                    }
                    TcpMessage::GroupMemberRemove {
                        ref group_id,
                        ref peer_id,
                    } => {
                        let d = db.lock().await;
                        let _ = d.remove_group_member(group_id, peer_id);
                    }
                }
            }
        }
    }

    // Connection closed — remove from connections
    if !remote_peer_id.is_empty() {
        let mut conns = state.connections.lock().await;
        conns.remove(&remote_peer_id);
    }
}

/// Connect to a peer's TCP server and return the write half.
/// Also spawns a reader task for incoming messages on this connection.
pub async fn connect_to_peer(
    peer_ip: &str,
    peer_tcp_port: u16,
    my_peer_id: &str,
    my_username: &str,
    state: Arc<SharedState>,
    db: Arc<TokioMutex<Database>>,
    proxy: EventLoopProxy<AppEvent>,
) -> Option<()> {
    let addr = format!("{peer_ip}:{peer_tcp_port}");
    let stream = TcpStream::connect(&addr).await.ok()?;
    let (rd, wr) = stream.into_split();

    // Send Hello
    let hello = TcpMessage::Hello {
        peer_id: my_peer_id.to_string(),
        username: my_username.to_string(),
    };
    let mut hello_json = serde_json::to_string(&hello).unwrap();
    hello_json.push('\n');

    let wr = tokio::sync::Mutex::new(wr);
    {
        let mut w = wr.lock().await;
        w.write_all(hello_json.as_bytes()).await.ok()?;
    }

    // We need to figure out the peer_id from the Hello response
    let mut reader = BufReader::new(rd);
    let mut first_line = String::new();
    reader.read_line(&mut first_line).await.ok()?;
    let hello_resp: TcpMessage = serde_json::from_str(first_line.trim()).ok()?;
    let remote_peer_id = match &hello_resp {
        TcpMessage::Hello { peer_id, .. } => peer_id.clone(),
        _ => return None,
    };

    // Store the writer in connections
    let wr = wr.into_inner();
    {
        let mut conns = state.connections.lock().await;
        conns.insert(remote_peer_id.clone(), wr);
    }

    // Spawn a reader for this connection
    let st = state.clone();
    let rpid = remote_peer_id.clone();
    tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let Ok(msg) = serde_json::from_str::<TcpMessage>(trimmed) else {
                        continue;
                    };
                    match msg {
                        TcpMessage::DirectMessage {
                            ref id,
                            ref from_id,
                            ref from_name,
                            ref content,
                            ref timestamp,
                        } => {
                            let row = MessageRow {
                                id: id.clone(),
                                conversation_id: from_id.clone(),
                                from_id: from_id.clone(),
                                from_name: from_name.clone(),
                                content: content.clone(),
                                timestamp: timestamp.clone(),
                                is_group: false,
                                status: "delivered".into(),
                            };
                            {
                                let d = db.lock().await;
                                let _ = d.insert_message(&row);
                            }
                            let js = js_call("incoming_message", &row);
                            let _ = proxy.send_event(AppEvent::EvalScript(js));
                        }
                        TcpMessage::GroupMessage {
                            ref id,
                            ref group_id,
                            ref from_id,
                            ref from_name,
                            ref content,
                            ref timestamp,
                        } => {
                            let row = MessageRow {
                                id: id.clone(),
                                conversation_id: group_id.clone(),
                                from_id: from_id.clone(),
                                from_name: from_name.clone(),
                                content: content.clone(),
                                timestamp: timestamp.clone(),
                                is_group: true,
                                status: "delivered".into(),
                            };
                            {
                                let d = db.lock().await;
                                let _ = d.insert_message(&row);
                            }
                            let js = js_call("incoming_message", &row);
                            let _ = proxy.send_event(AppEvent::EvalScript(js));
                        }
                        TcpMessage::Ack {
                            ref message_id,
                            ref status,
                        } => {
                            {
                                let d = db.lock().await;
                                let _ = d.update_message_status(message_id, status);
                            }
                            #[derive(serde::Serialize)]
                            struct AckInfo {
                                message_id: String,
                                status: String,
                            }
                            let js = js_call(
                                "message_ack",
                                &AckInfo {
                                    message_id: message_id.clone(),
                                    status: status.clone(),
                                },
                            );
                            let _ = proxy.send_event(AppEvent::EvalScript(js));
                        }
                        TcpMessage::GroupCreate {
                            ref group_id,
                            ref name,
                            ref creator_id,
                            ref members,
                        } => {
                            let d = db.lock().await;
                            let _ = d.create_group(group_id, name, creator_id);
                            for m in members {
                                let _ = d.add_group_member(group_id, m);
                            }
                            let groups = d.get_groups();
                            drop(d);
                            let js = js_call("group_list", &groups);
                            let _ = proxy.send_event(AppEvent::EvalScript(js));
                        }
                        _ => {}
                    }
                }
            }
        }
        // Connection closed
        let mut conns = st.connections.lock().await;
        conns.remove(&rpid);
    });

    Some(())
}

/// Send a TCP message to a specific peer. Ensures connection exists first.
pub async fn send_to_peer(
    peer_id: &str,
    msg: &TcpMessage,
    state: &SharedState,
) -> Result<(), String> {
    let mut conns = state.connections.lock().await;
    let writer = conns
        .get_mut(peer_id)
        .ok_or_else(|| "Not connected to peer".to_string())?;

    let mut data = serde_json::to_string(msg).map_err(|e| e.to_string())?;
    data.push('\n');
    writer
        .write_all(data.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

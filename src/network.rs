use crate::app_event::AppEvent;
use crate::db::{Database, MessageRow};
use crate::ipc::js_call;
use crate::protocol::TcpMessage;
use crate::state::{SharedState, SharedWriter};

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

    let writer: SharedWriter = Arc::new(TokioMutex::new(wr));
    {
        let mut w = writer.lock().await;
        if w.write_all(hello_json.as_bytes()).await.is_err() {
            return;
        }
    }

    // Read first message — must be Hello
    let mut first_line = String::new();
    match reader.read_line(&mut first_line).await {
        Ok(0) | Err(_) => return,
        Ok(_) => {}
    }
    let remote_peer_id = match serde_json::from_str::<TcpMessage>(first_line.trim()) {
        Ok(TcpMessage::Hello { peer_id, .. }) => peer_id,
        _ => return,
    };

    // Always register — newest connection wins (replaces dead ones too)
    {
        let mut conns = state.connections.lock().await;
        conns.insert(remote_peer_id.clone(), writer.clone());
    }

    // Read loop
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
                process_incoming(&msg, &writer, &db, &proxy).await;
            }
        }
    }

    // Connection closed — remove only if it's still OUR writer
    {
        let mut conns = state.connections.lock().await;
        if let Some(existing) = conns.get(&remote_peer_id) {
            if Arc::ptr_eq(existing, &writer) {
                conns.remove(&remote_peer_id);
            }
        }
    }
}

/// Connect to a peer's TCP server.
/// Stores the writer in state.connections and spawns a reader task.
pub async fn connect_to_peer(
    peer_ip: &str,
    peer_tcp_port: u16,
    my_peer_id: &str,
    my_username: &str,
    state: Arc<SharedState>,
    db: Arc<TokioMutex<Database>>,
    proxy: EventLoopProxy<AppEvent>,
) -> Result<(), String> {
    let addr = format!("{peer_ip}:{peer_tcp_port}");
    let stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("TCP connect to {addr}: {e}"))?;

    let (rd, wr) = stream.into_split();

    // Send Hello
    let hello = TcpMessage::Hello {
        peer_id: my_peer_id.to_string(),
        username: my_username.to_string(),
    };
    let mut hello_json = serde_json::to_string(&hello).unwrap();
    hello_json.push('\n');

    let writer: SharedWriter = Arc::new(TokioMutex::new(wr));
    {
        let mut w = writer.lock().await;
        w.write_all(hello_json.as_bytes())
            .await
            .map_err(|e| format!("Send hello: {e}"))?;
    }

    // Read Hello response
    let mut reader = BufReader::new(rd);
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .await
        .map_err(|e| format!("Read hello: {e}"))?;
    let remote_peer_id = match serde_json::from_str::<TcpMessage>(first_line.trim()) {
        Ok(TcpMessage::Hello { peer_id, .. }) => peer_id,
        _ => return Err("Bad hello response".into()),
    };

    // Store the writer — always overwrite (freshest connection)
    {
        let mut conns = state.connections.lock().await;
        conns.insert(remote_peer_id.clone(), writer.clone());
    }

    // Spawn reader task
    let st = state.clone();
    let rpid = remote_peer_id.clone();
    let wr_clone = writer.clone();
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
                    process_incoming(&msg, &wr_clone, &db, &proxy).await;
                }
            }
        }
        // Connection closed — remove only if it's still OUR writer
        {
            let mut conns = st.connections.lock().await;
            if let Some(existing) = conns.get(&rpid) {
                if Arc::ptr_eq(existing, &wr_clone) {
                    conns.remove(&rpid);
                }
            }
        }
    });

    Ok(())
}

/// Send a TCP message to a specific peer.
/// Returns Err if not connected or write fails.
pub async fn send_to_peer(
    peer_id: &str,
    msg: &TcpMessage,
    state: &SharedState,
) -> Result<(), String> {
    let writer = {
        let conns = state.connections.lock().await;
        conns
            .get(peer_id)
            .cloned()
            .ok_or_else(|| "Not connected to peer".to_string())?
    };

    let mut data = serde_json::to_string(msg).map_err(|e| e.to_string())?;
    data.push('\n');

    let mut w = writer.lock().await;
    w.write_all(data.as_bytes())
        .await
        .map_err(|e| format!("Write failed: {e}"))?;
    w.flush()
        .await
        .map_err(|e| format!("Flush failed: {e}"))?;
    Ok(())
}

/// Remove a dead connection from state so reconnect can happen.
pub async fn remove_connection(peer_id: &str, state: &SharedState) {
    let mut conns = state.connections.lock().await;
    conns.remove(peer_id);
}

// ── Shared message processing ──────────────────────────────────────

async fn process_incoming(
    msg: &TcpMessage,
    writer: &SharedWriter,
    db: &TokioMutex<Database>,
    proxy: &EventLoopProxy<AppEvent>,
) {
    match msg {
        TcpMessage::DirectMessage {
            id,
            from_id,
            from_name,
            content,
            timestamp,
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
            let _ = proxy.send_event(AppEvent::RequestAttention);
            send_ack(writer, id).await;
        }
        TcpMessage::GroupMessage {
            id,
            group_id,
            from_id,
            from_name,
            content,
            timestamp,
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
            let _ = proxy.send_event(AppEvent::RequestAttention);
            send_ack(writer, id).await;
        }
        TcpMessage::Ack {
            message_id,
            status,
        } => {
            {
                let d = db.lock().await;
                let _ = d.update_message_status(message_id, status);
            }
            #[derive(serde::Serialize)]
            struct AckInfo<'a> {
                message_id: &'a str,
                status: &'a str,
            }
            let js = js_call(
                "message_ack",
                &AckInfo {
                    message_id,
                    status,
                },
            );
            let _ = proxy.send_event(AppEvent::EvalScript(js));
        }
        TcpMessage::GroupCreate {
            group_id,
            name,
            creator_id,
            members,
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
        TcpMessage::GroupMemberAdd { group_id, peer_id } => {
            let d = db.lock().await;
            let _ = d.add_group_member(group_id, peer_id);
        }
        TcpMessage::GroupMemberRemove { group_id, peer_id } => {
            let d = db.lock().await;
            let _ = d.remove_group_member(group_id, peer_id);
        }
        TcpMessage::Hello { .. } => {}
    }
}

async fn send_ack(writer: &SharedWriter, message_id: &str) {
    let ack = TcpMessage::Ack {
        message_id: message_id.to_string(),
        status: "delivered".into(),
    };
    let mut ack_json = serde_json::to_string(&ack).unwrap();
    ack_json.push('\n');
    let mut w = writer.lock().await;
    let _ = w.write_all(ack_json.as_bytes()).await;
    let _ = w.flush().await;
}

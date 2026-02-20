use crate::protocol::UdpPacket;
use crate::state::{PeerInfo, SharedState};
use crate::ipc::js_call;
use crate::app_event::AppEvent;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type};
use tao::event_loop::EventLoopProxy;
use tokio::net::UdpSocket;

const DISCOVERY_PORT: u16 = 5555;
const ANNOUNCE_INTERVAL: Duration = Duration::from_secs(3);
const PEER_TIMEOUT: Duration = Duration::from_secs(10);

/// Start the UDP discovery system: announce ourselves + listen for others.
pub async fn run(
    peer_id: String,
    username: String,
    tcp_port: u16,
    state: Arc<SharedState>,
    proxy: EventLoopProxy<AppEvent>,
) {
    // Create a socket that can broadcast
    let socket = create_broadcast_socket().expect("Failed to create UDP socket");
    let socket = UdpSocket::from_std(socket.into()).expect("Failed to convert socket");
    let socket = Arc::new(socket);

    // Spawn the announce loop
    let s = socket.clone();
    let pid = peer_id.clone();
    let uname = username.clone();
    tokio::spawn(async move {
        let pkt = UdpPacket::Announce {
            peer_id: pid,
            username: uname,
            tcp_port,
        };
        let data = serde_json::to_vec(&pkt).unwrap();
        let broadcast_addr: SocketAddr =
            SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), DISCOVERY_PORT).into();

        loop {
            let _ = s.send_to(&data, &broadcast_addr).await;
            tokio::time::sleep(ANNOUNCE_INTERVAL).await;
        }
    });

    // Spawn the listener loop
    let s = socket.clone();
    let my_id = peer_id.clone();
    let st = state.clone();
    let px = proxy.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 2048];
        loop {
            let (len, addr) = match s.recv_from(&mut buf).await {
                Ok(r) => r,
                Err(_) => continue,
            };
            let Ok(pkt) = serde_json::from_slice::<UdpPacket>(&buf[..len]) else {
                continue;
            };
            match pkt {
                UdpPacket::Announce {
                    peer_id,
                    username,
                    tcp_port,
                } => {
                    if peer_id == my_id {
                        continue;
                    }
                    let ip = addr.ip().to_string();
                    let mut peers = st.peers.lock().await;
                    peers.insert(
                        peer_id.clone(),
                        PeerInfo {
                            peer_id,
                            username,
                            ip,
                            tcp_port,
                            last_seen: Instant::now(),
                        },
                    );
                    drop(peers);
                    send_peer_list(&st, &px).await;
                }
                UdpPacket::Goodbye { peer_id } => {
                    let mut peers = st.peers.lock().await;
                    peers.remove(&peer_id);
                    drop(peers);
                    send_peer_list(&st, &px).await;
                }
            }
        }
    });

    // Spawn the cleanup loop (remove stale peers)
    let st = state.clone();
    let px = proxy.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let mut peers = st.peers.lock().await;
            let before = peers.len();
            peers.retain(|_, p| p.last_seen.elapsed() < PEER_TIMEOUT);
            let after = peers.len();
            drop(peers);
            if after != before {
                send_peer_list(&st, &px).await;
            }
        }
    });

    // Send goodbye on drop (best-effort)
    let goodbye = UdpPacket::Goodbye {
        peer_id: peer_id.clone(),
    };
    let data = serde_json::to_vec(&goodbye).unwrap();
    let broadcast_addr: SocketAddr =
        SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), DISCOVERY_PORT).into();
    // Keep running until task is cancelled — goodbye sent from backend shutdown
    let _ = socket.send_to(&data, &broadcast_addr).await;
}

fn create_broadcast_socket() -> std::io::Result<std::net::UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;
    let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT);
    socket.bind(&addr.into())?;
    Ok(socket.into())
}

#[derive(serde::Serialize)]
struct PeerListItem {
    peer_id: String,
    username: String,
    ip: String,
}

pub async fn send_peer_list(state: &SharedState, proxy: &EventLoopProxy<AppEvent>) {
    let peers = state.peers.lock().await;
    let list: Vec<PeerListItem> = peers
        .values()
        .map(|p| PeerListItem {
            peer_id: p.peer_id.clone(),
            username: p.username.clone(),
            ip: p.ip.clone(),
        })
        .collect();
    drop(peers);
    let js = js_call("peer_list", &list);
    let _ = proxy.send_event(AppEvent::EvalScript(js));
}

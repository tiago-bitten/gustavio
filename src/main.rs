use std::io::{self, Write as _, stdout};
use std::sync::Arc;

use chrono::Local;
use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

// ── Cores ────────────────────────────────────────────────────

const CYAN: Color = Color::Rgb { r: 0, g: 255, b: 255 };
const GREEN: Color = Color::Rgb { r: 0, g: 255, b: 136 };
const DIM: Color = Color::Rgb { r: 100, g: 100, b: 100 };
const YELLOW: Color = Color::Rgb { r: 255, g: 217, b: 61 };
const RED: Color = Color::Rgb { r: 255, g: 107, b: 107 };
const WHITE: Color = Color::Rgb { r: 240, g: 240, b: 240 };
const GRAY: Color = Color::Rgb { r: 150, g: 150, b: 150 };
const SEP: Color = Color::Rgb { r: 60, g: 60, b: 60 };

const PALETTE: [Color; 8] = [
    Color::Rgb { r: 255, g: 107, b: 107 },
    Color::Rgb { r: 78, g: 205, b: 196 },
    Color::Rgb { r: 255, g: 217, b: 61 },
    Color::Rgb { r: 107, g: 137, b: 255 },
    Color::Rgb { r: 255, g: 107, b: 255 },
    Color::Rgb { r: 0, g: 255, b: 136 },
    Color::Rgb { r: 255, g: 159, b: 67 },
    Color::Rgb { r: 162, g: 155, b: 254 },
];

fn user_color(name: &str) -> Color {
    let h: usize = name.bytes().map(|b| b as usize).sum();
    PALETTE[h % PALETTE.len()]
}

// ── Protocolo ────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Msg {
    #[serde(rename = "t")]
    kind: String,
    #[serde(rename = "u")]
    user: String,
    #[serde(rename = "c", default)]
    content: String,
}

impl Msg {
    fn join(user: &str) -> Self {
        Self { kind: "join".into(), user: user.into(), content: String::new() }
    }
    fn leave(user: &str) -> Self {
        Self { kind: "leave".into(), user: user.into(), content: String::new() }
    }
    fn chat(user: &str, text: &str) -> Self {
        Self { kind: "msg".into(), user: user.into(), content: text.into() }
    }
}

// ── UI helpers ───────────────────────────────────────────────

fn banner() {
    let _ = execute!(
        stdout(),
        Print("\n"),
        SetForegroundColor(CYAN),
        Print("  ╔═══════════════════════════════════════════╗\n"),
        Print("  ║                                           ║\n"),
        Print("  ║         "),
        SetForegroundColor(Color::White),
        SetAttribute(Attribute::Bold),
        Print("G U S T A V I O   C H A T"),
        SetAttribute(Attribute::Reset),
        SetForegroundColor(CYAN),
        Print("       ║\n"),
        Print("  ║                                           ║\n"),
        Print("  ╚═══════════════════════════════════════════╝\n\n"),
        SetForegroundColor(DIM),
        Print("  Chat no terminal via rede local\n"),
        Print("  Digite "),
        SetForegroundColor(YELLOW),
        Print("/sair"),
        SetForegroundColor(DIM),
        Print(" para sair\n\n"),
        ResetColor,
    );
}

fn separator() {
    let _ = execute!(
        stdout(),
        SetForegroundColor(SEP),
        Print("  ─────────────────────────────────────────────────\n\n"),
        ResetColor,
    );
}

fn display(msg: &Msg) {
    let t = Local::now().format("%H:%M:%S");
    let mut out = stdout();
    match msg.kind.as_str() {
        "join" => {
            let _ = execute!(
                out,
                SetForegroundColor(DIM),
                Print(format!("  [{t}] ")),
                SetForegroundColor(YELLOW),
                Print(">> "),
                SetForegroundColor(user_color(&msg.user)),
                SetAttribute(Attribute::Bold),
                Print(&msg.user),
                SetAttribute(Attribute::Reset),
                SetForegroundColor(YELLOW),
                Print(" entrou no chat\n"),
                ResetColor,
            );
        }
        "leave" => {
            let _ = execute!(
                out,
                SetForegroundColor(DIM),
                Print(format!("  [{t}] ")),
                SetForegroundColor(RED),
                Print("<< "),
                SetForegroundColor(user_color(&msg.user)),
                SetAttribute(Attribute::Bold),
                Print(&msg.user),
                SetAttribute(Attribute::Reset),
                SetForegroundColor(RED),
                Print(" saiu do chat\n"),
                ResetColor,
            );
        }
        "msg" => {
            let _ = execute!(
                out,
                SetForegroundColor(DIM),
                Print(format!("  [{t}] ")),
                SetForegroundColor(user_color(&msg.user)),
                SetAttribute(Attribute::Bold),
                Print(&msg.user),
                SetAttribute(Attribute::Reset),
                SetForegroundColor(GRAY),
                Print(": "),
                SetForegroundColor(WHITE),
                Print(&msg.content),
                Print("\n"),
                ResetColor,
            );
        }
        _ => {}
    }
}

fn prompt() {
    let _ = execute!(
        stdout(),
        SetForegroundColor(GREEN),
        Print("  > "),
        ResetColor,
    );
    let _ = stdout().flush();
}

fn ask(label: &str) -> String {
    let _ = execute!(
        stdout(),
        SetForegroundColor(CYAN),
        Print(format!("  {label}")),
        ResetColor,
    );
    let _ = stdout().flush();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
}

// ── Servidor ─────────────────────────────────────────────────

async fn run_server(username: String, port: u16) {
    let addr = format!("0.0.0.0:{port}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("  Erro ao iniciar servidor: {e}");
            return;
        }
    };

    // Mostrar IP local
    if let Ok(ip) = local_ip_address::local_ip() {
        let _ = execute!(
            stdout(),
            SetForegroundColor(GREEN),
            SetAttribute(Attribute::Bold),
            Print(format!("  Sala criada! Conecte em: {ip}:{port}\n\n")),
            SetAttribute(Attribute::Reset),
            ResetColor,
        );
    } else {
        let _ = execute!(
            stdout(),
            SetForegroundColor(GREEN),
            Print(format!("  Sala criada na porta {port}\n\n")),
            ResetColor,
        );
    }

    separator();

    // Canal de broadcast: (sender_tag, json)
    let (tx, _) = broadcast::channel::<(String, String)>(256);
    let tx = Arc::new(tx);

    // Anunciar entrada do host
    let join = Msg::join(&username);
    display(&join);
    let _ = tx.send(("HOST".into(), serde_json::to_string(&join).unwrap()));

    // Aceitar conexões
    let tx_accept = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok((stream, addr)) = listener.accept().await {
                let tag = addr.to_string();
                let tx = tx_accept.clone();
                let rx = tx.subscribe();
                tokio::spawn(relay(stream, tag, tx, rx));
            }
        }
    });

    // Receptor do host (mensagens de clientes)
    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok((tag, json)) => {
                    if tag != "HOST" {
                        if let Ok(m) = serde_json::from_str::<Msg>(&json) {
                            display(&m);
                            prompt();
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Loop de input do host
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    prompt();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            prompt();
            continue;
        }
        if line == "/sair" || line == "/quit" {
            break;
        }

        let m = Msg::chat(&username, &line);
        display(&m);
        let _ = tx.send(("HOST".into(), serde_json::to_string(&m).unwrap()));
        prompt();
    }

    // Sair
    let leave = Msg::leave(&username);
    display(&leave);
    let _ = tx.send(("HOST".into(), serde_json::to_string(&leave).unwrap()));
    // Dar tempo para o broadcast ser enviado
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
}

/// Relay de um cliente conectado ao servidor.
/// Lê mensagens do cliente e transmite para todos;
/// recebe do broadcast e envia para o cliente (exceto as dele mesmo).
async fn relay(
    stream: TcpStream,
    tag: String,
    tx: Arc<broadcast::Sender<(String, String)>>,
    mut rx: broadcast::Receiver<(String, String)>,
) {
    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd);

    let username = Arc::new(tokio::sync::Mutex::new(String::new()));
    let uname_for_disconnect = username.clone();

    // Writer: envia broadcast para este cliente (menos as mensagens dele)
    let my_tag = tag.clone();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok((sender, json)) => {
                    if sender != my_tag {
                        let mut data = json;
                        data.push('\n');
                        if wr.write_all(data.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Reader: lê do cliente e faz broadcast
    let mut line = String::new();
    loop {
        line.clear();
        match rd.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let json = line.trim().to_string();
                if json.is_empty() {
                    continue;
                }
                // Guardar username para mensagem de saída
                if let Ok(m) = serde_json::from_str::<Msg>(&json) {
                    if m.kind == "join" {
                        *username.lock().await = m.user.clone();
                    }
                }
                let _ = tx.send((tag.clone(), json));
            }
        }
    }

    // Cliente desconectou
    let name = uname_for_disconnect.lock().await.clone();
    if !name.is_empty() {
        let leave = Msg::leave(&name);
        let _ = tx.send((tag, serde_json::to_string(&leave).unwrap()));
    }
}

// ── Cliente ──────────────────────────────────────────────────

async fn run_client(username: String, addr: String) {
    let stream = match TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            let _ = execute!(
                stdout(),
                SetForegroundColor(RED),
                Print(format!("  Erro ao conectar: {e}\n")),
                ResetColor,
            );
            return;
        }
    };

    let _ = execute!(
        stdout(),
        SetForegroundColor(GREEN),
        SetAttribute(Attribute::Bold),
        Print(format!("  Conectado a {addr}!\n\n")),
        SetAttribute(Attribute::Reset),
        ResetColor,
    );
    separator();

    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd);

    // Enviar join
    let join = Msg::join(&username);
    display(&join);
    let mut json = serde_json::to_string(&join).unwrap();
    json.push('\n');
    if wr.write_all(json.as_bytes()).await.is_err() {
        eprintln!("  Erro ao enviar join");
        return;
    }

    // Receptor: lê do servidor e exibe
    tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            match rd.read_line(&mut line).await {
                Ok(0) | Err(_) => {
                    let _ = execute!(
                        stdout(),
                        Print("\n"),
                        SetForegroundColor(RED),
                        SetAttribute(Attribute::Bold),
                        Print("  Desconectado do servidor.\n"),
                        SetAttribute(Attribute::Reset),
                        ResetColor,
                    );
                    std::process::exit(0);
                }
                Ok(_) => {
                    if let Ok(m) = serde_json::from_str::<Msg>(line.trim()) {
                        display(&m);
                        prompt();
                    }
                }
            }
        }
    });

    // Loop de input
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    prompt();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            prompt();
            continue;
        }
        if line == "/sair" || line == "/quit" {
            break;
        }

        let m = Msg::chat(&username, &line);
        display(&m);
        let mut json = serde_json::to_string(&m).unwrap();
        json.push('\n');
        if wr.write_all(json.as_bytes()).await.is_err() {
            let _ = execute!(
                stdout(),
                SetForegroundColor(RED),
                Print("  Erro ao enviar mensagem\n"),
                ResetColor,
            );
            break;
        }
        prompt();
    }

    // Enviar leave
    let leave = Msg::leave(&username);
    let mut json = serde_json::to_string(&leave).unwrap();
    json.push('\n');
    let _ = wr.write_all(json.as_bytes()).await;
}

// ── Main ─────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    banner();

    let _ = execute!(
        stdout(),
        SetForegroundColor(GREEN),
        Print("  [1] "),
        ResetColor,
        Print("Criar sala (servidor)\n"),
        SetForegroundColor(GREEN),
        Print("  [2] "),
        ResetColor,
        Print("Entrar em sala (cliente)\n\n"),
    );

    let mode = ask("Opção: ");
    let username = ask("Seu nome: ");

    if username.is_empty() {
        let _ = execute!(
            stdout(),
            SetForegroundColor(RED),
            Print("\n  Nome não pode ser vazio!\n"),
            ResetColor,
        );
        return;
    }

    println!();

    match mode.as_str() {
        "1" => run_server(username, 9999).await,
        "2" => {
            let addr = ask("Endereço (ex: 192.168.1.100:9999): ");
            println!();
            run_client(username, addr).await;
        }
        _ => {
            let _ = execute!(
                stdout(),
                SetForegroundColor(RED),
                Print("  Opção inválida!\n"),
                ResetColor,
            );
        }
    }
}

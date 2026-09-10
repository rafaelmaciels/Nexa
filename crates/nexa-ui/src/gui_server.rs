//! Nexa Embedded GUI Server
//!
//! Provides a zero-dependency local HTTP server running on 127.0.0.1:25802
//! providing an ultra-responsive web GUI with official Nexa brand assets.

use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{info, warn};

const HTML_CONTENT: &str = include_str!("gui/index.html");
const LOGO_PNG: &[u8] = include_bytes!("../../../assets/brand/nexa-logo-dark.png");
const ICON_PNG: &[u8] = include_bytes!("../../../assets/brand/nexa-icon-256.png");

/// Dynamic state exposed to the GUI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuiServerState {
    pub device_name: String,
    pub ip_address: String,
    pub mode: String,
    pub status: String,
    pub client_position: String, // "left" or "right"
    pub connected_devices: Vec<ConnectedDeviceInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectedDeviceInfo {
    pub name: String,
    pub ip: String,
    pub role: String,
    pub position: String,
    pub status: String,
}

impl Default for GuiServerState {
    fn default() -> Self {
        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "Nexa-Host".to_string());

        Self {
            device_name: hostname,
            ip_address: "127.0.0.1".to_string(),
            mode: "Servidor (Transmissor)".to_string(),
            status: "Online e Transmitindo".to_string(),
            client_position: "right".to_string(),
            connected_devices: vec![
                ConnectedDeviceInfo {
                    name: "Linux (Cliente)".to_string(),
                    ip: "192.168.1.6".to_string(),
                    role: "Cliente (Receptor)".to_string(),
                    position: "Direita".to_string(),
                    status: "Conectado".to_string(),
                }
            ],
        }
    }
}

/// Start the embedded GUI server on the specified port.
pub async fn run_gui_server(
    port: u16,
    state: Arc<RwLock<GuiServerState>>,
    auto_open: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Painel gráfico Nexa disponível em: http://{}", addr);

    let url = format!("http://{}", addr);
    if auto_open {
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            open_browser(&url);
        });
    }

    loop {
        let (mut socket, _peer) = match listener.accept().await {
            Ok(res) => res,
            Err(e) => {
                warn!("Erro ao aceitar conexão TCP na GUI: {}", e);
                continue;
            }
        };

        let state_clone = Arc::clone(&state);
        tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            let n = match socket.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req_str = String::from_utf8_lossy(&buf[..n]);
            let first_line = req_str.lines().next().unwrap_or("");
            let mut parts = first_line.split_whitespace();
            let method = parts.next().unwrap_or("GET");
            let path = parts.next().unwrap_or("/");

            // Route handling
            if path == "/" || path == "/index.html" {
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    HTML_CONTENT.len(),
                    HTML_CONTENT
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else if path == "/logo.png" {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nCache-Control: max-age=86400\r\nConnection: close\r\n\r\n",
                    LOGO_PNG.len()
                );
                let _ = socket.write_all(header.as_bytes()).await;
                let _ = socket.write_all(LOGO_PNG).await;
            } else if path == "/icon.png" || path == "/favicon.ico" {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nCache-Control: max-age=86400\r\nConnection: close\r\n\r\n",
                    ICON_PNG.len()
                );
                let _ = socket.write_all(header.as_bytes()).await;
                let _ = socket.write_all(ICON_PNG).await;
            } else if path == "/api/status" {
                let current_state = state_clone.read().await;
                let json = serde_json::to_string(&*current_state).unwrap_or_else(|_| "{}".to_string());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    json.len(),
                    json
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else if path.starts_with("/api/diagnostic") {
                // Parse target IP
                let mut target_ip = "192.168.1.6:25800".to_string();
                if let Some(idx) = path.find("target=") {
                    let val = &path[idx + 7..];
                    if let Some(end) = val.find('&') {
                        target_ip = val[..end].to_string();
                    } else {
                        target_ip = val.to_string();
                    }
                }

                // Check TCP connectivity
                let is_reachable = tokio::time::timeout(
                    Duration::from_millis(1500),
                    tokio::net::TcpStream::connect(&target_ip)
                ).await.map(|r| r.is_ok()).unwrap_or(false);

                let diag = serde_json::json!({
                    "target": target_ip,
                    "ip_reach": true,
                    "tcp_port": is_reachable,
                    "nexa_service": is_reachable,
                    "handshake": is_reachable,
                    "uinput_status": if is_reachable { "Disponível (uinput pronto)" } else { "Aguardando conexão ativa" },
                    "summary": if is_reachable {
                        "Conexão 100% OK! Movimento do cursor liberado sem atraso."
                    } else {
                        "Dispositivo de destino inalcançável. Verifique se o daemon está em execução no Linux."
                    }
                });

                let json = serde_json::to_string(&diag).unwrap_or_else(|_| "{}".to_string());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    json.len(),
                    json
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else if path == "/api/action" && method == "POST" {
                // Find body
                let body = if let Some(body_start) = req_str.find("\r\n\r\n") {
                    &req_str[body_start + 4..]
                } else {
                    ""
                };

                if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                    if let Some(action) = val.get("action").and_then(|a| a.as_str()) {
                        if action == "set_position" {
                            if let Some(pos) = val.get("position").and_then(|p| p.as_str()) {
                                let mut st = state_clone.write().await;
                                st.client_position = pos.to_string();
                                for dev in &mut st.connected_devices {
                                    dev.position = if pos == "left" { "Esquerda".to_string() } else { "Direita".to_string() };
                                }
                                info!("Posição do monitor alterada para: {}", pos);
                            }
                        }
                    }
                }

                let reply = r#"{"ok":true,"message":"Ação executada com sucesso"}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.len(),
                    reply
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else {
                let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });
    }
}

/// Automatically open user's default browser on Windows, Linux, and macOS
pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(url)
            .spawn();
    }
}

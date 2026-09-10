use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{error, info, warn};
use nexa_common::NexaError;
use nexa_core::{AppConfig, ScreenGeometry, SessionEngine, DEFAULT_PORT};
use nexa_core::fsm::SessionState;
use nexa_crypto::MachineIdentity;
use nexa_net::{NexaClient, NexaDiscoveryService, NexaServer, DEFAULT_DISCOVERY_PORT};
use nexa_platform::traits::{
    CapturedInputEvent, ClipboardManager, InputCapturer, ScreenManager,
};
use nexa_protocol::{KeyState, NexaPacket};

#[cfg(windows)]
use nexa_platform::{
    WindowsClipboardManager as PlatformClipboardManager,
    WindowsDisplayManager as PlatformScreenManager,
    WindowsInputCapturer as PlatformCapturer,
    WindowsInputInjector as PlatformInjector,
};

#[cfg(not(windows))]
use nexa_platform::{
    LinuxClipboardManager as PlatformClipboardManager,
    LinuxDisplayManager as PlatformScreenManager,
    LinuxInputCapturer as PlatformCapturer,
    LinuxInputInjector as PlatformInjector,
};

/// Configuração de inicialização do Daemon
#[derive(Debug, Clone)]
pub struct DaemonOptions {
    pub config_path: PathBuf,
    pub is_server_mode: bool,
    pub listen_port: u16,
    pub enable_discovery: bool,
    pub connect_target: Option<String>,
}

impl Default for DaemonOptions {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("nexa.toml"),
            is_server_mode: true,
            listen_port: DEFAULT_PORT,
            enable_discovery: true,
            connect_target: None,
        }
    }
}

/// Orquestrador de serviços em background do Nexa
pub struct NexaDaemonService {
    options: DaemonOptions,
    config: AppConfig,
    identity: MachineIdentity,
    is_running: Arc<AtomicBool>,
}

impl NexaDaemonService {
    /// Inicializa o daemon carregando ou gerando chaves de identidade e configuração local
    pub fn new(options: DaemonOptions) -> Result<Self, NexaError> {
        let config = if options.config_path.exists() {
            info!("Carregando configuração existente de {:?}", options.config_path);
            AppConfig::load_from_file(&options.config_path)?
        } else {
            info!("Criando configuração padrão em {:?}", options.config_path);
            let cfg = AppConfig::default();
            let _ = cfg.save_to_file(&options.config_path);
            cfg
        };

        // Identidade Ed25519 de longo prazo
        let identity = MachineIdentity::generate();
        info!(
            "Identidade do computador inicializada: Nome='{}', Chave Pública={}",
            config.device_name,
            &identity.public_key_hex()[..16]
        );

        Ok(Self {
            options,
            config,
            identity,
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn device_name(&self) -> &str {
        &self.config.device_name
    }

    pub fn public_key_hex(&self) -> String {
        self.identity.public_key_hex()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Executa o ciclo de vida dos serviços em segundo plano
    pub async fn start(&self) -> Result<(), NexaError> {
        self.is_running.store(true, Ordering::SeqCst);
        info!("Nexa Daemon v{} iniciado!", env!("CARGO_PKG_VERSION"));

        // 1. Inicia o serviço de descoberta local (UDP Broadcast)
        if self.options.enable_discovery {
            let discovery = Arc::new(NexaDiscoveryService::new(
                &self.config.device_name,
                self.options.listen_port,
                &self.identity.public_key_hex(),
            ));

            let disc_clone = Arc::clone(&discovery);
            let running = Arc::clone(&self.is_running);
            tokio::spawn(async move {
                let target = format!("255.255.255.255:{}", DEFAULT_DISCOVERY_PORT);
                while running.load(Ordering::SeqCst) {
                    let _ = disc_clone.send_announcement(&target).await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
            });

            info!("Descoberta LAN ativa na porta {} (anúncio a cada 3s).", DEFAULT_DISCOVERY_PORT);
        }

        // 2. Determina o modo de operação (Servidor Host ou Cliente Guest)
        if self.options.is_server_mode && self.options.connect_target.is_none() {
            info!(
                "Modo Servidor (Host) ativado. Escutando na porta TCP {}...",
                self.options.listen_port
            );

            let server = NexaServer::bind(&format!("0.0.0.0:{}", self.options.listen_port)).await?;
            let is_running = Arc::clone(&self.is_running);
            let device_name = self.config.device_name.clone();
            let edge_delay_ms = self.config.edge_delay_ms;
            let peers = self.config.peers.clone();

            tokio::spawn(async move {
                Self::run_server_loop(server, device_name, edge_delay_ms, peers, is_running).await;
            });
        } else {
            let target = self.options.connect_target.clone().unwrap_or_else(|| {
                self.config
                    .peers
                    .first()
                    .map(|p| {
                        format!(
                            "{}:{}",
                            p.manual_ip.as_deref().unwrap_or(&p.ip_or_host),
                            p.manual_port.unwrap_or(p.port)
                        )
                    })
                    .unwrap_or_else(|| format!("127.0.0.1:{}", DEFAULT_PORT))
            });

            info!("Modo Cliente (Guest) ativado com destino: {}", target);

            let is_running = Arc::clone(&self.is_running);
            let device_name = self.config.device_name.clone();

            tokio::spawn(async move {
                Self::run_client_loop(target, device_name, is_running).await;
            });
        }

        Ok(())
    }

    /// Loop de execução do Servidor (Host)
    async fn run_server_loop(
        server: NexaServer,
        device_name: String,
        edge_delay_ms: u64,
        peers: Vec<nexa_core::PeerConfig>,
        is_running: Arc<AtomicBool>,
    ) {
        #[cfg(windows)]
        nexa_platform::enable_dpi_awareness();

        while is_running.load(Ordering::SeqCst) {
            info!("Servidor pronto. Aguardando conexão de outro computador na rede...");

            let (mut secure_conn, sas_pin) = match server.accept_handshake().await {
                Ok(res) => res,
                Err(e) => {
                    if !is_running.load(Ordering::SeqCst) {
                        break;
                    }
                    warn!("Erro no handshake de entrada: {}. Aguardando nova tentativa...", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
            };

            info!("┌──────────────────────────────────────────────────┐");
            info!("│           NOVO COMPUTADOR CONECTADO              │");
            info!("├──────────────────────────────────────────────────┤");
            info!("│  Código de Pareamento de Segurança (SAS PIN):   │");
            info!("│                     {}                      │", sas_pin);
            info!("│                                                  │");
            info!("│  Verifique se o código acima é idêntico na tela  │");
            info!("│  do outro computador para evitar ataques MITM.   │");
            info!("└──────────────────────────────────────────────────┘");

            // Instancia capturador de periféricos locais
            let mut capturer = PlatformCapturer::new();
            if let Err(e) = capturer.start() {
                error!("Falha ao inicializar hook de captura de periféricos: {}", e);
                continue;
            }

            let screen_mgr = PlatformScreenManager::default();
            let clip_mgr = PlatformClipboardManager::default();
            let (screen_x, screen_y, screen_w, screen_h) = match screen_mgr.get_screen_bounds() {
                Ok((x, y, w, h)) if w > 0 && h > 0 => (x, y, w as u32, h as u32),
                _ => (0, 0, 1920, 1080),
            };

            let mut engine = SessionEngine::new(
                &device_name,
                ScreenGeometry::with_origin(screen_x, screen_y, screen_w, screen_h),
                edge_delay_ms.max(50), // Garante atraso mínimo saudável para evitar transições acidentais
            );

            // Registra a tela do cliente na topologia (padrão: à direita da tela principal)
            let remote_name = peers
                .first()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Guest-Screen".to_string());

            engine.topology_mut().register_screen(&remote_name, ScreenGeometry::new(1920, 1080));
            engine.topology_mut().link_horizontal(&device_name, &remote_name);

            info!("Topologia de telas configurada: Mova o mouse para a BORDA DIREITA da tela para entrar no Linux.");

            let receiver = capturer.receiver().clone();
            let (tx, mut rx) = tokio::sync::mpsc::channel::<NexaPacket>(512);

            let session_active = Arc::new(AtomicBool::new(true));
            let session_active_clone = Arc::clone(&session_active);
            let is_running_clone = Arc::clone(&is_running);
            let capturer_arc = Arc::new(capturer);
            let capturer_worker = Arc::clone(&capturer_arc);

            // Worker em thread separada para desempilhar eventos do hook com latência sub-milissegundo
            let event_worker_handle = std::thread::Builder::new()
                .name("nexa-event-pump".into())
                .spawn(move || {
                    while session_active_clone.load(Ordering::SeqCst) && is_running_clone.load(Ordering::SeqCst) {
                        match receiver.recv_timeout(std::time::Duration::from_millis(15)) {
                            Ok(event) => {
                                match event {
                                    CapturedInputEvent::MouseMove { x, y } => {
                                        let is_remote = matches!(engine.fsm().current_state(), SessionState::RemoteActive { .. });
                                        if let Ok(Some(pkt)) = engine.handle_local_mouse_move(x, y, is_remote) {
                                            match engine.fsm().current_state() {
                                                SessionState::RemoteActive { .. } => {
                                                    capturer_worker.set_suppression(true);
                                                    if matches!(pkt, NexaPacket::ScreenEnter(_)) {
                                                        info!(">>> [KVM] Cursor cruzou para o Linux! (Pressione ESC a qualquer momento para voltar)");
                                                        if let Ok(Some(clip_pkt)) = engine.sync_clipboard_on_transition(&clip_mgr) {
                                                            let _ = tx.try_send(clip_pkt);
                                                        }
                                                    }
                                                }
                                                SessionState::LocalActive => {
                                                    // Transição de retorno suave para a máquina local (Windows)
                                                    capturer_worker.set_suppression(false);
                                                    info!("<<< [KVM] Cursor retornou para o Windows!");
                                                    let (vx, vy, vw, vh) = match screen_mgr.get_screen_bounds() {
                                                        Ok(b) => b,
                                                        Err(_) => (0, 0, 1920, 1080),
                                                    };
                                                    let return_x = vx + vw - 120;
                                                    let return_y = (engine.virtual_remote_y() as i32 + vy).clamp(vy + 50, vy + vh - 50);
                                                    let _ = screen_mgr.set_cursor_position(return_x, return_y);
                                                }
                                                _ => {}
                                            }
                                            let _ = tx.try_send(pkt);
                                        }
                                    }
                                    CapturedInputEvent::MouseButton { button, is_down } => {
                                        if let Some(pkt) = engine.handle_local_mouse_button(button, is_down) {
                                            let _ = tx.try_send(pkt);
                                        }
                                    }
                                    CapturedInputEvent::MouseWheel { delta_x, delta_y } => {
                                        if let Some(pkt) = engine.handle_local_mouse_wheel(delta_x, delta_y) {
                                            let _ = tx.try_send(pkt);
                                        }
                                    }
                                    CapturedInputEvent::Key { scancode, state } => {
                                        // Teclas de emergência para retornar o controle à tela local:
                                        // ESC (0x0001) ou Scroll Lock (0x0046)
                                        if (scancode == 0x0001 || scancode == 0x0046) && state == KeyState::Down {
                                            if matches!(engine.fsm().current_state(), SessionState::RemoteActive { .. }) {
                                                engine.fsm_mut().on_return_to_local();
                                                capturer_worker.set_suppression(false);
                                                let _ = tx.try_send(NexaPacket::ScreenLeave(nexa_protocol::ScreenLeave { timestamp_ms: 0 }));
                                                info!("Cursor devolvido à tela local via tecla de emergência (ESC / ScrollLock).");
                                                let (vx, vy, vw, vh) = match screen_mgr.get_screen_bounds() {
                                                    Ok(b) => b,
                                                    Err(_) => (0, 0, 1920, 1080),
                                                };
                                                let _ = screen_mgr.set_cursor_position(vx + vw / 2, vy + vh / 2);
                                            }
                                        } else if let Some(pkt) = engine.handle_local_key(scancode, state, 0) {
                                            let _ = tx.try_send(pkt);
                                        }
                                    }
                                }
                            }
                            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                        }
                    }
                });

            // Loop de comunicação assíncrona com o cliente
            while is_running.load(Ordering::SeqCst) {
                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                        if !is_running.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                    maybe_pkt = rx.recv() => {
                        match maybe_pkt {
                            Some(pkt) => {
                                if let Err(e) = secure_conn.send_packet(&pkt).await {
                                    warn!("Falha de envio TCP para o cliente: {}. Desconectando.", e);
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    recv_res = secure_conn.recv_packet() => {
                        match recv_res {
                            Ok(Some(remote_pkt)) => {
                                match remote_pkt {
                                    NexaPacket::ScreenLeave(_) => {
                                        capturer_arc.set_suppression(false);
                                    }
                                    NexaPacket::Clipboard(clip) => {
                                        let clip_mgr = PlatformClipboardManager::default();
                                        let _ = clip_mgr.set_text(&clip.text);
                                    }
                                    _ => {}
                                }
                            }
                            Ok(None) => {
                                info!("Cliente desconectou voluntariamente.");
                                break;
                            }
                            Err(e) => {
                                warn!("Conexão com cliente interrompida: {}.", e);
                                break;
                            }
                        }
                    }
                }
            }

            session_active.store(false, Ordering::SeqCst);
            capturer_arc.set_suppression(false);
            if let Ok(handle) = event_worker_handle {
                let _ = handle.join();
            }
            drop(capturer_arc);
            info!("Conexão finalizada. Servidor aguarda nova conexão.");
        }
    }

    /// Loop de execução do Cliente (Guest)
    async fn run_client_loop(
        target_addr: String,
        device_name: String,
        is_running: Arc<AtomicBool>,
    ) {
        while is_running.load(Ordering::SeqCst) {
            info!("Tentando conectar ao servidor Nexa em {}...", target_addr);

            match NexaClient::connect(&target_addr).await {
                Ok((mut secure_conn, sas_pin)) => {
                    info!("┌──────────────────────────────────────────────────┐");
                    info!("│          CONEXÃO ESTABELECIDA COM SUCESSO        │");
                    info!("├──────────────────────────────────────────────────┤");
                    info!("│  Código de Pareamento de Segurança (SAS PIN):   │");
                    info!("│                     {}                      │", sas_pin);
                    info!("│                                                  │");
                    info!("│  Confirme se o código acima é idêntico na tela   │");
                    info!("│  do servidor.                                    │");
                    info!("└──────────────────────────────────────────────────┘");

                    let injector = PlatformInjector::default();
                    let screen_mgr = PlatformScreenManager::default();
                    let clip_mgr = PlatformClipboardManager::default();
                    let (screen_w, screen_h) = match screen_mgr.get_screen_bounds() {
                        Ok((_, _, w, h)) if w > 0 && h > 0 => (w as u32, h as u32),
                        _ => (1920, 1080),
                    };

                    let mut engine = SessionEngine::new(
                        &device_name,
                        ScreenGeometry::new(screen_w, screen_h),
                        100,
                    );

                    while is_running.load(Ordering::SeqCst) {
                        tokio::select! {
                            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                                if !is_running.load(Ordering::SeqCst) {
                                    break;
                                }
                            }
                            packet_res = secure_conn.recv_packet() => {
                                match packet_res {
                                    Ok(Some(packet)) => {
                                        if matches!(packet, NexaPacket::ScreenEnter(_)) {
                                            info!(">>> [KVM CLIENTE] Cursor e teclado recebidos do host Windows!");
                                        } else if matches!(packet, NexaPacket::ScreenLeave(_)) {
                                            info!("<<< [KVM CLIENTE] Cursor devolvido ao host Windows.");
                                        }
                                        if let Err(e) = engine.handle_remote_packet_with_clipboard(
                                            &packet,
                                            &injector,
                                            &screen_mgr,
                                            &clip_mgr,
                                        ) {
                                            warn!("Erro ao injetar comando remoto: {}", e);
                                        }
                                    }
                                    Ok(None) => {
                                        info!("O servidor encerrou a sessão.");
                                        break;
                                    }
                                    Err(e) => {
                                        warn!("Conexão com o servidor perdida: {}.", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Falha ao conectar em {}: {}. Tentando novamente em 3 segundos...",
                        target_addr, e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
            }
        }
    }

    /// Encerramento gracioso do daemon
    pub fn stop(&self) {
        if self.is_running.swap(false, Ordering::SeqCst) {
            info!("Encerramento do Nexa Daemon solicitado. Desinstalando hooks e liberando periféricos...");
        }
    }
}

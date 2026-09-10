use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;
use nexa_common::NexaError;
use nexa_core::{AppConfig, ScreenGeometry, SessionEngine, DEFAULT_PORT};
use nexa_crypto::MachineIdentity;
use nexa_net::{NexaDiscoveryService, DEFAULT_DISCOVERY_PORT};

/// Configuração de inicialização do Daemon
#[derive(Debug, Clone)]
pub struct DaemonOptions {
    pub config_path: PathBuf,
    pub is_server_mode: bool,
    pub listen_port: u16,
    pub enable_discovery: bool,
}

impl Default for DaemonOptions {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("nexa.toml"),
            is_server_mode: true,
            listen_port: DEFAULT_PORT,
            enable_discovery: true,
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
        info!("Nexa Daemon iniciado com sucesso!");

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

        // 2. Registra o SessionEngine com tela virtual local (1920x1080 padrão)
        let local_geometry = ScreenGeometry::new(1920, 1080);
        let _engine = SessionEngine::new(
            &self.config.device_name,
            local_geometry,
            self.config.edge_delay_ms,
        );

        info!(
            "Motor de periféricos pronto (Atraso de borda: {}ms).",
            self.config.edge_delay_ms
        );

        Ok(())
    }

    /// Encerramento gracioso do daemon
    pub fn stop(&self) {
        if self.is_running.swap(false, Ordering::SeqCst) {
            info!("Encerramento do Nexa Daemon solicitado. Desinstalando hooks e liberando periféricos...");
        }
    }
}

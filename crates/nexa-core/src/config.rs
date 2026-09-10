use serde::{Deserialize, Serialize};
use std::path::Path;
use nexa_common::NexaError;

pub const DEFAULT_PORT: u16 = 25800;
pub const DEFAULT_EDGE_DELAY_MS: u64 = 100;

/// Configuração de uma máquina pareada/autorizada ou configurada manualmente
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerConfig {
    pub name: String,
    pub ip_or_host: String,
    pub port: u16,
    pub public_key_hex: String,
    pub is_trusted: bool,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub manual_ip: Option<String>,
    #[serde(default)]
    pub manual_port: Option<u16>,
    #[serde(default)]
    pub last_seen_ip: Option<String>,
}

/// Configuração principal da aplicação Nexa
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppConfig {
    pub device_name: String,
    pub listen_port: u16,
    pub edge_delay_ms: u64,
    pub auto_connect: bool,
    pub log_level: String,
    pub preferred_interface: Option<String>,
    pub peers: Vec<PeerConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_name: hostname_fallback(),
            listen_port: DEFAULT_PORT,
            edge_delay_ms: DEFAULT_EDGE_DELAY_MS,
            auto_connect: true,
            log_level: "info".to_string(),
            preferred_interface: None,
            peers: Vec::new(),
        }
    }
}

impl AppConfig {
    /// Carrega configuração a partir de um arquivo TOML
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, NexaError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| NexaError::Config(format!("Falha ao ler arquivo de configuração: {}", e)))?;
        toml::from_str(&content)
            .map_err(|e| NexaError::Config(format!("Arquivo de configuração inválido: {}", e)))
    }

    /// Salva a configuração atual em formato TOML
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), NexaError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| NexaError::Config(format!("Falha ao serializar configuração: {}", e)))?;
        std::fs::write(path, content)
            .map_err(|e| NexaError::Config(format!("Falha ao escrever arquivo de configuração: {}", e)))
    }

    /// Atualiza o IP associado a um computador já confiável com base na sua chave pública Ed25519
    /// Suporta transição transparente de IP dinâmico (DHCP) sem perder a confiança prévia
    pub fn update_peer_ip_if_trusted(&mut self, public_key_hex: &str, new_ip: &str) -> bool {
        for peer in &mut self.peers {
            if peer.public_key_hex == public_key_hex && peer.is_trusted {
                peer.last_seen_ip = Some(peer.ip_or_host.clone());
                peer.ip_or_host = new_ip.to_string();
                return true;
            }
        }
        false
    }

    /// Adiciona ou atualiza a definição manual de um computador por IP e porta
    pub fn add_or_update_manual_peer(
        &mut self,
        ip: &str,
        port: u16,
        alias: Option<&str>,
        public_key_hex: Option<&str>,
    ) {
        let name = alias.unwrap_or(ip).to_string();
        if let Some(existing) = self.peers.iter_mut().find(|p| p.ip_or_host == ip || p.manual_ip.as_deref() == Some(ip)) {
            existing.port = port;
            existing.manual_ip = Some(ip.to_string());
            existing.manual_port = Some(port);
            if let Some(a) = alias {
                existing.alias = Some(a.to_string());
                existing.name = a.to_string();
            }
            if let Some(pk) = public_key_hex {
                existing.public_key_hex = pk.to_string();
            }
        } else {
            self.peers.push(PeerConfig {
                name,
                ip_or_host: ip.to_string(),
                port,
                public_key_hex: public_key_hex.unwrap_or("").to_string(),
                is_trusted: public_key_hex.is_some(),
                alias: alias.map(|s| s.to_string()),
                manual_ip: Some(ip.to_string()),
                manual_port: Some(port),
                last_seen_ip: None,
            });
        }
    }

    /// Remove um computador cadastrado por nome, alias ou endereço IP
    pub fn remove_peer(&mut self, identifier: &str) -> bool {
        let original_len = self.peers.len();
        self.peers.retain(|p| {
            p.name != identifier
                && p.alias.as_deref() != Some(identifier)
                && p.ip_or_host != identifier
                && p.manual_ip.as_deref() != Some(identifier)
                && p.public_key_hex != identifier
        });
        self.peers.len() < original_len
    }

    /// Verifica se uma chave pública Ed25519 já foi previamente autorizada e marcada como confiável
    pub fn is_peer_trusted(&self, public_key_hex: &str) -> bool {
        if public_key_hex.is_empty() {
            return false;
        }
        self.peers.iter().any(|p| p.public_key_hex == public_key_hex && p.is_trusted)
    }
}

fn hostname_fallback() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "Nexa-Host".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_save_load_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("nexa.toml");

        let mut cfg = AppConfig::default();
        cfg.device_name = "Meu-PC-Windows".to_string();
        cfg.listen_port = 24850;
        cfg.peers.push(PeerConfig {
            name: "Linux-Elementary".to_string(),
            ip_or_host: "192.168.1.50".to_string(),
            port: 25800,
            public_key_hex: "abcdef1234567890".to_string(),
            is_trusted: true,
            alias: None,
            manual_ip: None,
            manual_port: None,
            last_seen_ip: None,
        });

        cfg.save_to_file(&config_path).expect("Falha ao salvar");
        let loaded = AppConfig::load_from_file(&config_path).expect("Falha ao carregar");

        assert_eq!(cfg, loaded);
        assert_eq!(loaded.peers.len(), 1);
        assert_eq!(loaded.peers[0].name, "Linux-Elementary");
    }

    #[test]
    fn test_partial_config_fallback_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("partial_nexa.toml");

        // Simula arquivo com apenas device_name e listen_port definidos
        let partial_toml = r#"
device_name = "Meu-PC"
listen_port = 25800
"#;
        std::fs::write(&config_path, partial_toml).unwrap();

        let loaded = AppConfig::load_from_file(&config_path).expect("Deveria carregar com defaults");
        assert_eq!(loaded.device_name, "Meu-PC");
        assert_eq!(loaded.listen_port, 25800);
        assert!(loaded.auto_connect);
        assert_eq!(loaded.log_level, "info");
        assert_eq!(loaded.edge_delay_ms, DEFAULT_EDGE_DELAY_MS);
        assert!(loaded.peers.is_empty());
    }
}

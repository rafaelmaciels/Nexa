use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

use nexa_common::NexaError;

pub const DEFAULT_DISCOVERY_PORT: u16 = 25855;

/// Informações de um computador descoberto na rede local
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredDevice {
    pub device_name: String,
    pub service_port: u16,
    pub public_key_hex: String,
    pub ip_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscoveryBeacon {
    name: String,
    port: u16,
    pk: String,
}

/// Serviço de anúncio e escuta de presença na rede local via UDP
pub struct NexaDiscoveryService {
    device_name: String,
    service_port: u16,
    public_key_hex: String,
    discovered: Arc<Mutex<HashMap<String, DiscoveredDevice>>>,
}

impl NexaDiscoveryService {
    pub fn new(
        device_name: impl Into<String>,
        service_port: u16,
        public_key_hex: impl Into<String>,
    ) -> Self {
        Self {
            device_name: device_name.into(),
            service_port,
            public_key_hex: public_key_hex.into(),
            discovered: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Retorna a lista de dispositivos descobertos até o momento
    pub fn get_discovered_devices(&self) -> Vec<DiscoveredDevice> {
        let map = self.discovered.lock().unwrap();
        map.values().cloned().collect()
    }

    /// Executa um único ciclo de envio de anúncio para um endereço alvo (broadcast ou unicast)
    pub async fn send_announcement(&self, target: &str) -> Result<(), NexaError> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao criar socket de anúncio: {}", e)))?;
        let _ = socket.set_broadcast(true);

        let beacon = DiscoveryBeacon {
            name: self.device_name.clone(),
            port: self.service_port,
            pk: self.public_key_hex.clone(),
        };

        let data = serde_json::to_vec(&beacon)
            .map_err(|e| NexaError::Protocol(format!("Falha ao serializar beacon: {}", e)))?;

        socket
            .send_to(&data, target)
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao enviar beacon de descoberta: {}", e)))?;

        Ok(())
    }

    /// Processa um pacote de anúncio recebido
    pub fn process_incoming_packet(&self, data: &[u8], src: SocketAddr) -> Result<Option<DiscoveredDevice>, NexaError> {
        let beacon: DiscoveryBeacon = match serde_json::from_slice(data) {
            Ok(b) => b,
            Err(_) => return Ok(None),
        };

        // Não registra anúncios da própria máquina
        if beacon.pk == self.public_key_hex {
            return Ok(None);
        }

        let device = DiscoveredDevice {
            device_name: beacon.name,
            service_port: beacon.port,
            public_key_hex: beacon.pk.clone(),
            ip_address: src.ip().to_string(),
        };

        let mut map = self.discovered.lock().unwrap();
        map.insert(beacon.pk, device.clone());

        Ok(Some(device))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_announcement_and_processing() {
        // Máquina A (Host Windows)
        let service_a = NexaDiscoveryService::new("Meu-PC-Windows", 24850, "pk_host_windows_11");

        // Máquina B (Client Linux)
        let service_b = NexaDiscoveryService::new("Linux-Laptop", 24850, "pk_client_linux_elementary");

        // Listener socket na porta livre
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let listen_addr = listener.local_addr().unwrap();

        // Máquina B anuncia para o endereço do listener
        service_b.send_announcement(&listen_addr.to_string()).await.unwrap();

        // Listener recebe o pacote
        let mut buf = [0u8; 1024];
        let (len, src) = listener.recv_from(&mut buf).await.unwrap();

        // Máquina A processa o pacote
        let discovered_opt = service_a.process_incoming_packet(&buf[..len], src).unwrap();
        assert!(discovered_opt.is_some());

        let discovered = discovered_opt.unwrap();
        assert_eq!(discovered.device_name, "Linux-Laptop");
        assert_eq!(discovered.service_port, 24850);
        assert_eq!(discovered.public_key_hex, "pk_client_linux_elementary");

        // Lista de dispositivos descobertos na Máquina A
        let all = service_a.get_discovered_devices();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].device_name, "Linux-Laptop");
    }
}

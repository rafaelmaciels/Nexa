use std::net::UdpSocket;
use serde::{Deserialize, Serialize};

/// Informações sobre uma interface de rede local disponível no computador
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalNetworkInterface {
    pub name: String,
    pub ip_address: String,
    pub is_ipv6: bool,
    pub is_loopback: bool,
}

/// Detecta a interface e IP de saída padrão conectando um socket UDP não-bloqueante
pub fn detect_primary_local_ip() -> Option<String> {
    // Tenta determinar a interface de saída principal através do mecanismo de rota do SO
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

/// Enumera as interfaces e endereços IP conhecidos da máquina
pub fn detect_local_interfaces() -> Vec<LocalNetworkInterface> {
    let mut interfaces = Vec::new();

    // 1. Loopback padrão
    interfaces.push(LocalNetworkInterface {
        name: "Loopback Local".to_string(),
        ip_address: "127.0.0.1".to_string(),
        is_ipv6: false,
        is_loopback: true,
    });

    // 2. IP Primário Ativo (Wi-Fi ou Ethernet)
    if let Some(primary_ip) = detect_primary_local_ip() {
        if primary_ip != "127.0.0.1" {
            let is_ipv6 = primary_ip.contains(':');
            interfaces.push(LocalNetworkInterface {
                name: "Rede Local (Wi-Fi / Ethernet)".to_string(),
                ip_address: primary_ip,
                is_ipv6,
                is_loopback: false,
            });
        }
    }

    interfaces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_interfaces_detection() {
        let list = detect_local_interfaces();
        assert!(!list.is_empty(), "Deve detectar ao menos o loopback local");
        assert!(list.iter().any(|i| i.ip_address == "127.0.0.1"));
    }
}

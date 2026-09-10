use serde::{Deserialize, Serialize};
use nexa_net::{ConnectionDiagnostic, DiscoveredDevice, LocalNetworkInterface};

/// Método de conexão preferencial
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionMode {
    AutoDiscovery,
    ManualIp,
}

/// Status visual da conexão
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Pairing,
    Disconnected,
}

impl ConnectionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ConnectionStatus::Connected => "🟢 Conectado",
            ConnectionStatus::Pairing => "🟡 Pareando...",
            ConnectionStatus::Disconnected => "⚪ Desconectado",
        }
    }
}

/// Estado do formulário modal de adição manual de computador por IP
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddComputerFormState {
    pub is_visible: bool,
    pub ip_address: String,
    pub port: u16,
    pub alias: String,
    pub diagnostic: Option<ConnectionDiagnostic>,
    pub is_testing: bool,
}

impl Default for AddComputerFormState {
    fn default() -> Self {
        Self {
            is_visible: false,
            ip_address: String::new(),
            port: nexa_core::DEFAULT_PORT,
            alias: String::new(),
            diagnostic: None,
            is_testing: false,
        }
    }
}

/// Dados do diálogo modal de confirmação de pareamento visual
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingModal {
    pub is_visible: bool,
    pub remote_name: String,
    pub remote_ip: String,
    pub sas_pin: String, // ex: "482 731"
}

impl Default for PairingModal {
    fn default() -> Self {
        Self {
            is_visible: false,
            remote_name: String::new(),
            remote_ip: String::new(),
            sas_pin: String::new(),
        }
    }
}

/// Dispositivo exibido na lista da interface
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDevice {
    pub name: String,
    pub ip: String,
    pub is_local: bool,
    pub is_trusted: bool,
    pub is_manual: bool,
}

/// Estado global da interface do usuário
pub struct UiState {
    pub local_device: UiDevice,
    pub connection_mode: ConnectionMode,
    pub local_interfaces: Vec<LocalNetworkInterface>,
    pub status: ConnectionStatus,
    pub connected_peers: Vec<UiDevice>,
    pub discovered_peers: Vec<DiscoveredDevice>,
    pub pairing_modal: Option<PairingModal>,
    pub add_computer_modal: Option<AddComputerFormState>,
}

impl UiState {
    pub fn new(local_name: impl Into<String>, local_ip: impl Into<String>) -> Self {
        let ip_str = local_ip.into();
        Self {
            local_device: UiDevice {
                name: local_name.into(),
                ip: ip_str.clone(),
                is_local: true,
                is_trusted: true,
                is_manual: false,
            },
            connection_mode: ConnectionMode::AutoDiscovery,
            local_interfaces: nexa_net::detect_local_interfaces(),
            status: ConnectionStatus::Disconnected,
            connected_peers: Vec::new(),
            discovered_peers: Vec::new(),
            pairing_modal: None,
            add_computer_modal: None,
        }
    }

    pub fn set_connection_mode(&mut self, mode: ConnectionMode) {
        self.connection_mode = mode;
    }

    /// Abre a janela modal de Adicionar Computador Manual por IP
    pub fn open_add_computer(&mut self) {
        self.add_computer_modal = Some(AddComputerFormState {
            is_visible: true,
            ip_address: "192.168.1.20".to_string(),
            port: nexa_core::DEFAULT_PORT,
            alias: String::new(),
            diagnostic: None,
            is_testing: false,
        });
    }

    pub fn close_add_computer(&mut self) {
        self.add_computer_modal = None;
    }

    /// Atualiza o formulário com o relatório de diagnóstico do teste de conexão
    pub fn set_diagnostic_result(&mut self, diag: ConnectionDiagnostic) {
        if let Some(modal) = &mut self.add_computer_modal {
            modal.diagnostic = Some(diag);
            modal.is_testing = false;
        }
    }

    /// Adiciona o computador configurado à lista de conexões conhecidas
    pub fn confirm_add_computer(&mut self) -> Option<UiDevice> {
        let modal = self.add_computer_modal.take()?;
        let display_name = if modal.alias.trim().is_empty() {
            modal.ip_address.clone()
        } else {
            modal.alias.clone()
        };

        let device = UiDevice {
            name: display_name,
            ip: format!("{}:{}", modal.ip_address, modal.port),
            is_local: false,
            is_trusted: false,
            is_manual: true,
        };

        self.connected_peers.push(device.clone());
        Some(device)
    }

    /// Abre o diálogo de pareamento com o código SAS de 6 dígitos formatado
    pub fn show_pairing_request(
        &mut self,
        remote_name: impl Into<String>,
        remote_ip: impl Into<String>,
        sas_pin: impl Into<String>,
    ) {
        self.status = ConnectionStatus::Pairing;
        self.pairing_modal = Some(PairingModal {
            is_visible: true,
            remote_name: remote_name.into(),
            remote_ip: remote_ip.into(),
            sas_pin: sas_pin.into(),
        });
    }

    /// Usuário clica em [Aceitar]
    pub fn accept_pairing(&mut self) -> Option<PairingModal> {
        let modal = self.pairing_modal.take()?;
        self.status = ConnectionStatus::Connected;
        self.connected_peers.push(UiDevice {
            name: modal.remote_name.clone(),
            ip: modal.remote_ip.clone(),
            is_local: false,
            is_trusted: true,
            is_manual: false,
        });
        Some(modal)
    }

    /// Usuário clica em [Recusar]
    pub fn reject_pairing(&mut self) {
        self.pairing_modal = None;
        self.status = ConnectionStatus::Disconnected;
    }

    /// Atualiza a lista de computadores descobertos na rede local
    pub fn update_discovered(&mut self, devices: Vec<DiscoveredDevice>) {
        self.discovered_peers = devices;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_workflow_accept() {
        let mut state = UiState::new("Meu-PC", "192.168.1.10");
        assert_eq!(state.status, ConnectionStatus::Disconnected);

        // Solicitação de pareamento recebida
        state.show_pairing_request("Linux-Laptop", "192.168.1.20", "482 731");
        assert_eq!(state.status, ConnectionStatus::Pairing);
        assert!(state.pairing_modal.is_some());
        assert_eq!(state.pairing_modal.as_ref().unwrap().sas_pin, "482 731");

        // Usuário aceita
        let modal = state.accept_pairing().unwrap();
        assert_eq!(modal.remote_name, "Linux-Laptop");
        assert_eq!(state.status, ConnectionStatus::Connected);
        assert_eq!(state.connected_peers.len(), 1);
        assert_eq!(state.connected_peers[0].name, "Linux-Laptop");
    }

    #[test]
    fn test_pairing_workflow_reject() {
        let mut state = UiState::new("Meu-PC", "192.168.1.10");
        state.show_pairing_request("Invasor-Desconhecido", "192.168.1.99", "123 456");

        state.reject_pairing();
        assert_eq!(state.status, ConnectionStatus::Disconnected);
        assert!(state.pairing_modal.is_none());
        assert_eq!(state.connected_peers.len(), 0);
    }
}

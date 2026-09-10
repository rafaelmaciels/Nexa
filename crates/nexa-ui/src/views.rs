use crate::layout_canvas::VisualLayoutManager;
use crate::state::{AddComputerFormState, PairingModal, UiState};
use nexa_net::ConnectionDiagnostic;

/// Renderizador textual / componente de apresentação da UI
pub struct UiViewRenderer;

impl UiViewRenderer {
    /// Renderiza o diálogo modal de Adicionar Computador Manual por IP
    pub fn render_add_computer_dialog(form: &AddComputerFormState) -> String {
        let mut out = String::new();
        out.push_str("┌──────────────────────────────────────┐\n");
        out.push_str("│       Adicionar computador           │\n");
        out.push_str("├──────────────────────────────────────┤\n");
        out.push_str("│                                      │\n");
        out.push_str("│ Endereço IP                          │\n");
        out.push_str("│ ┌──────────────────────────────────┐ │\n");
        out.push_str(&format!("│ │ {:<32} │ │\n", form.ip_address));
        out.push_str("│ └──────────────────────────────────┘ │\n");
        out.push_str("│                                      │\n");
        out.push_str("│ Porta                                │\n");
        out.push_str("│ ┌──────────────────────────────────┐ │\n");
        out.push_str(&format!("│ │ {:<32} │ │\n", form.port));
        out.push_str("│ └──────────────────────────────────┘ │\n");
        out.push_str("│                                      │\n");
        out.push_str("│ Nome opcional                        │\n");
        out.push_str("│ ┌──────────────────────────────────┐ │\n");
        out.push_str(&format!("│ │ {:<32} │ │\n", form.alias));
        out.push_str("│ └──────────────────────────────────┘ │\n");
        out.push_str("│                                      │\n");
        out.push_str("│ [ Testar conexão ]                   │\n");
        out.push_str("│ [ Adicionar ]                        │\n");
        out.push_str("└──────────────────────────────────────┘\n");
        out
    }

    /// Renderiza o painel de Diagnóstico de Rede passo a passo
    pub fn render_diagnostic_panel(diag: &ConnectionDiagnostic) -> String {
        let mut out = String::new();
        out.push_str("┌────────────────────────────────────────────┐\n");
        out.push_str("│ Diagnóstico de conexão                     │\n");
        out.push_str("├────────────────────────────────────────────┤\n");
        out.push_str("│                                            │\n");
        out.push_str("│ Destino                                    │\n");
        out.push_str(&format!("│ {:<42} │\n", diag.target_addr));
        out.push_str("│                                            │\n");
        out.push_str(&format!(
            "│ Conectividade IP             {:<13} │\n",
            diag.ip_connectivity.symbol()
        ));
        out.push_str(&format!(
            "│ Porta TCP                    {:<13} │\n",
            diag.tcp_port_open.symbol()
        ));
        out.push_str(&format!(
            "│ Serviço Nexa                 {:<13} │\n",
            diag.nexa_service.symbol()
        ));
        out.push_str(&format!(
            "│ Handshake                    {:<13} │\n",
            diag.crypto_handshake.symbol()
        ));
        out.push_str(&format!(
            "│ Autenticação                 {:<13} │\n",
            diag.authorization.symbol()
        ));
        out.push_str("│                                            │\n");
        out.push_str("│ Resultado                                  │\n");
        let res_icon = if diag.overall_success { "✓" } else { "✗" };
        out.push_str(&format!("│ {} {:<39} │\n", res_icon, diag.diagnosis_summary));

        if !diag.possible_causes.is_empty() {
            out.push_str("│                                            │\n");
            out.push_str("│ Possíveis causas:                          │\n");
            for cause in &diag.possible_causes {
                out.push_str(&format!("│ • {:<40} │\n", cause));
            }
        }
        out.push_str("└────────────────────────────────────────────┘\n");
        out
    }
    /// Renderiza o diálogo modal de confirmação visual do código SAS de 6 dígitos
    pub fn render_pairing_modal(modal: &PairingModal) -> String {
        format!(
            "Novo computador encontrado\n\n\
             Nome: {}\n\
             Endereço: {}\n\n\
             Código de pareamento:\n\n\
                   {}\n\n\
             [ Aceitar ]    [ Recusar ]",
            modal.remote_name, modal.remote_ip, modal.sas_pin
        )
    }

    /// Renderiza o painel principal da aplicação com a lista de dispositivos e layout
    pub fn render_main_window(state: &UiState, layout: &VisualLayoutManager) -> String {
        let mut out = String::new();
        out.push_str("┌────────────────────────────────────────────┐\n");
        out.push_str("│  Nexa - Compartilhamento de Teclado/Mouse  │\n");
        out.push_str("├────────────────────────────────────────────┤\n");
        out.push_str("│                                            │\n");
        out.push_str("│  Este computador                           │\n");
        out.push_str(&format!(
            "│  💻 {} ({})\n",
            state.local_device.name, state.local_device.ip
        ));

        // Mostra interfaces locais detectadas (Wi-Fi, Ethernet, etc.)
        for iface in &state.local_interfaces {
            if !iface.is_loopback {
                out.push_str(&format!("│     └─ {}: {}\n", iface.name, iface.ip_address));
            }
        }

        out.push_str("│                                            │\n");
        out.push_str("│  Computadores                              │\n");

        if state.connected_peers.is_empty() {
            out.push_str("│  (Nenhum computador conectado no momento)  │\n");
        } else {
            for peer in &state.connected_peers {
                let badge = if peer.is_trusted { "🟢" } else { "🟡" };
                out.push_str(&format!("│  {} {} ({})\n", badge, peer.name, peer.ip));
            }
        }

        out.push_str("│                                            │\n");
        out.push_str("│  [+ Adicionar computador]                  │\n");
        out.push_str("│                                            │\n");
        out.push_str("│  Método de conexão:                        │\n");
        match state.connection_mode {
            crate::state::ConnectionMode::AutoDiscovery => {
                out.push_str("│  ● Descoberta automática                   │\n");
                out.push_str("│  ○ Configuração manual                     │\n");
            }
            crate::state::ConnectionMode::ManualIp => {
                out.push_str("│  ○ Descoberta automática                   │\n");
                out.push_str("│  ● Configuração manual                     │\n");
            }
        }

        out.push_str("│                                            │\n");
        out.push_str("│  Layout de Telas                           │\n");

        if let Some(placements) = layout.get_placements(&state.local_device.name) {
            for (pos, target) in placements {
                out.push_str(&format!(
                    "│  [{}] {:?} [{}]\n",
                    state.local_device.name, pos, target
                ));
            }
        } else {
            out.push_str(&format!("│  [{}]\n", state.local_device.name));
        }

        out.push_str("│                                            │\n");
        out.push_str(&format!("│  Status: {:<32}  │\n", state.status.label()));
        out.push_str("└────────────────────────────────────────────┘\n");

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout_canvas::ScreenPosition;

    #[test]
    fn test_render_pairing_modal_output() {
        let modal = PairingModal {
            is_visible: true,
            remote_name: "DESKTOP-SALA".to_string(),
            remote_ip: "192.168.1.20".to_string(),
            sas_pin: "482 731".to_string(),
        };

        let rendered = UiViewRenderer::render_pairing_modal(&modal);
        assert!(rendered.contains("DESKTOP-SALA"));
        assert!(rendered.contains("192.168.1.20"));
        assert!(rendered.contains("482 731"));
        assert!(rendered.contains("[ Aceitar ]"));
        assert!(rendered.contains("[ Recusar ]"));
    }

    #[test]
    fn test_render_main_window_output() {
        let state = UiState::new("Meu-PC", "192.168.1.10");
        let mut layout = VisualLayoutManager::new();
        layout.place_screen("Linux-Laptop", ScreenPosition::Right, "Meu-PC");

        let rendered = UiViewRenderer::render_main_window(&state, &layout);
        assert!(rendered.contains("Nexa"));
        assert!(rendered.contains("Meu-PC"));
        assert!(rendered.contains("192.168.1.10"));
        assert!(rendered.contains("Linux-Laptop"));
    }
}

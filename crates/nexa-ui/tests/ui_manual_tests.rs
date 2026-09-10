use nexa_net::{ConnectionDiagnostic, DiagnosticStepStatus};
use nexa_ui::{ConnectionMode, UiState, UiViewRenderer, VisualLayoutManager};

#[test]
fn test_ui_add_manual_computer_flow() {
    let mut state = UiState::new("Meu-PC", "192.168.1.10");

    // 1. Abre modal de adicionar computador
    state.open_add_computer();
    assert!(state.add_computer_modal.is_some());
    let modal = state.add_computer_modal.as_mut().unwrap();
    assert_eq!(modal.port, 25800);

    // 2. Preenche os campos
    modal.ip_address = "192.168.1.20".to_string();
    modal.alias = "Meu Linux".to_string();

    // 3. Renderiza o diálogo
    let rendered_dialog = UiViewRenderer::render_add_computer_dialog(modal);
    assert!(rendered_dialog.contains("Adicionar computador"));
    assert!(rendered_dialog.contains("192.168.1.20"));
    assert!(rendered_dialog.contains("25800"));
    assert!(rendered_dialog.contains("Meu Linux"));
    assert!(rendered_dialog.contains("[ Testar conexão ]"));
    assert!(rendered_dialog.contains("[ Adicionar ]"));

    // 4. Confirma adição
    let added = state.confirm_add_computer().expect("Deveria adicionar peer");
    assert_eq!(added.name, "Meu Linux");
    assert_eq!(added.ip, "192.168.1.20:25800");
    assert!(added.is_manual);
    assert_eq!(state.connected_peers.len(), 1);
}

#[test]
fn test_ui_diagnostic_panel_rendering() {
    let mut diag = ConnectionDiagnostic::new_initial("192.168.1.20:25800");
    diag.ip_connectivity = DiagnosticStepStatus::Success;
    diag.tcp_port_open = DiagnosticStepStatus::Failed("Conexão recusada (Connection Refused)".into());
    diag.nexa_service = DiagnosticStepStatus::Skipped;
    diag.crypto_handshake = DiagnosticStepStatus::Skipped;
    diag.authorization = DiagnosticStepStatus::Skipped;
    diag.diagnosis_summary = "Host alcançável, mas a porta do Nexa está fechada.".into();
    diag.possible_causes = vec![
        "O Nexa não está em execução no computador de destino.".into(),
        "O firewall bloqueou a conexão.".into(),
    ];

    let rendered = UiViewRenderer::render_diagnostic_panel(&diag);
    assert!(rendered.contains("Diagnóstico de conexão"));
    assert!(rendered.contains("192.168.1.20:25800"));
    assert!(rendered.contains("Conectividade IP             ✓"));
    assert!(rendered.contains("Porta TCP                    ✗"));
    assert!(rendered.contains("Serviço Nexa                 ?"));
    assert!(rendered.contains("Possíveis causas:"));
    assert!(rendered.contains("O Nexa não está em execução"));
}

#[test]
fn test_ui_main_window_with_interfaces_and_mode() {
    let mut state = UiState::new("Meu-PC", "192.168.1.10");
    state.set_connection_mode(ConnectionMode::ManualIp);

    let layout = VisualLayoutManager::new();
    let rendered = UiViewRenderer::render_main_window(&state, &layout);

    assert!(rendered.contains("Nexa"));
    assert!(rendered.contains("● Configuração manual"));
    assert!(rendered.contains("○ Descoberta automática"));
    assert!(rendered.contains("[+ Adicionar computador]"));
}

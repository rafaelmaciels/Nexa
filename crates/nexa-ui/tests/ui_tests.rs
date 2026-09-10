use nexa_core::{EdgeSide, ScreenGeometry, SessionEngine};
use nexa_net::DiscoveredDevice;
use nexa_ui::{ConnectionStatus, ScreenPosition, UiState, UiViewRenderer, VisualLayoutManager};

#[test]
fn test_end_to_end_ui_discovery_and_pairing_acceptance() {
    let mut ui = UiState::new("Meu-PC", "192.168.1.10");
    assert_eq!(ui.status, ConnectionStatus::Disconnected);

    // 1. Simula descoberta de rede local
    ui.update_discovered(vec![DiscoveredDevice {
        device_name: "DESKTOP-SALA".to_string(),
        service_port: 24850,
        public_key_hex: "abcdef123456".to_string(),
        ip_address: "192.168.1.20".to_string(),
    }]);
    assert_eq!(ui.discovered_peers.len(), 1);

    // 2. Dispara solicitação de pareamento com PIN visual SAS de 6 dígitos
    ui.show_pairing_request("DESKTOP-SALA", "192.168.1.20", "482 731");
    assert_eq!(ui.status, ConnectionStatus::Pairing);

    // 3. Renderiza o modal e valida o conteúdo em conformidade com os requisitos
    let modal_ref = ui.pairing_modal.as_ref().unwrap();
    let modal_rendered = UiViewRenderer::render_pairing_modal(modal_ref);
    assert!(modal_rendered.contains("DESKTOP-SALA"));
    assert!(modal_rendered.contains("192.168.1.20"));
    assert!(modal_rendered.contains("482 731"));
    assert!(modal_rendered.contains("[ Aceitar ]"));
    assert!(modal_rendered.contains("[ Recusar ]"));

    // 4. Usuário aceita o pareamento
    let accepted = ui.accept_pairing();
    assert!(accepted.is_some());
    assert_eq!(ui.status, ConnectionStatus::Connected);
    assert_eq!(ui.connected_peers.len(), 1);
    assert_eq!(ui.connected_peers[0].name, "DESKTOP-SALA");
}

#[test]
fn test_end_to_end_layout_to_engine_sync() {
    let mut layout = VisualLayoutManager::new();

    // Organiza telas: Linux à esquerda, Windows no centro, Laptop à direita
    layout.place_screen("Linux-Elementary", ScreenPosition::Left, "Windows-PC");
    layout.place_screen("Laptop-Secundario", ScreenPosition::Right, "Windows-PC");

    // Motor de sessão do Windows
    let mut engine = SessionEngine::new("Windows-PC", ScreenGeometry::new(1920, 1080), 0);
    engine
        .topology_mut()
        .register_screen("Linux-Elementary", ScreenGeometry::new(2560, 1440));
    engine
        .topology_mut()
        .register_screen("Laptop-Secundario", ScreenGeometry::new(1920, 1080));

    // Sincroniza o layout visual da interface para a topologia do Core
    layout.sync_to_topology(engine.topology_mut());

    // Valida que a borda esquerda do Windows leva ao Linux
    let left_neighbor = engine
        .topology_mut()
        .get_neighbor("Windows-PC", EdgeSide::Left)
        .expect("Deve ter vizinho à esquerda");
    assert_eq!(left_neighbor.target_screen_id, "Linux-Elementary");

    // Valida que a borda direita do Windows leva ao Laptop
    let right_neighbor = engine
        .topology_mut()
        .get_neighbor("Windows-PC", EdgeSide::Right)
        .expect("Deve ter vizinho à direita");
    assert_eq!(right_neighbor.target_screen_id, "Laptop-Secundario");
}

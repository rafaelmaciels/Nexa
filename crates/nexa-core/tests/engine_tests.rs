use nexa_core::{
    MouseCoalescer, ScreenGeometry, SessionEngine, SessionState,
};
use nexa_platform::{LinuxDisplayManager, LinuxInputInjector};
use nexa_protocol::{
    KeyState, MouseButton, MouseMotionMode, MouseMove, NexaPacket, ScreenLeave,
};
use std::time::Duration;

#[test]
fn test_mouse_coalescer_consolidation() {
    let mut coalescer = MouseCoalescer::new(Duration::from_millis(5));

    // Injeta 10 micromovimentos
    for _ in 0..10 {
        coalescer.push(MouseMove {
            mode: MouseMotionMode::Relative,
            x: 3,
            y: 2,
        });
    }

    let consolidated = coalescer.flush().expect("Deve consolidar deltas acumulados");
    assert_eq!(consolidated.mode, MouseMotionMode::Relative);
    assert_eq!(consolidated.x, 30); // 10 * 3 = 30
    assert_eq!(consolidated.y, 20); // 10 * 2 = 20
}

#[test]
fn test_bidirectional_session_engine_simulation() {
    // 1. Cria o SessionEngine do Host (Windows 1920x1080)
    let win_geo = ScreenGeometry::new(1920, 1080);
    let mut host_engine = SessionEngine::new("windows_host", win_geo, 0); // 0ms para comutação instantânea no teste

    // Configura o vizinho Linux (elementary OS 2560x1440) à direita
    host_engine
        .topology_mut()
        .register_screen("linux_client", ScreenGeometry::new(2560, 1440));
    host_engine
        .topology_mut()
        .link_horizontal("windows_host", "linux_client");

    // Mock do nó receptor (Linux)
    let linux_injector = LinuxInputInjector::new();
    let mut linux_display = LinuxDisplayManager::new();
    linux_display.set_dimensions(2560, 1440);

    // Estado inicial: LocalActive
    assert_eq!(host_engine.fsm().current_state(), &SessionState::LocalActive);

    // 2. Cursor atinge a borda direita (X=1919, Y=540)
    let enter_packet_opt = host_engine
        .handle_local_mouse_move(1919, 540, false)
        .expect("Processar movimento");

    let enter_packet = enter_packet_opt.expect("Deve gerar ScreenEnter ao cruzar a borda");
    assert!(matches!(enter_packet, NexaPacket::ScreenEnter(_)));
    assert_eq!(
        host_engine.fsm().current_state(),
        &SessionState::RemoteActive {
            active_target: "linux_client".into()
        }
    );

    // 3. Nó receptor Linux recebe e processa o ScreenEnter
    let res = host_engine.handle_remote_packet(&enter_packet, &linux_injector, &linux_display);
    assert!(res.is_ok());

    // 4. Host emite eventos físicos enquanto em modo remoto
    let move_pkt = host_engine
        .handle_local_mouse_move(15, -10, true)
        .unwrap()
        .expect("Deve gerar MouseMove");
    assert!(matches!(move_pkt, NexaPacket::MouseMove(_)));

    let click_pkt = host_engine
        .handle_local_mouse_button(MouseButton::Left, true)
        .expect("Deve gerar MouseButton");
    assert!(matches!(click_pkt, NexaPacket::MouseButton(_)));

    let key_pkt = host_engine
        .handle_local_key(0x1E, KeyState::Down, 0)
        .expect("Deve gerar KeyEvent");
    assert!(matches!(key_pkt, NexaPacket::KeyEvent(_)));

    // 5. Injeção de todos os eventos no nó Linux receptor
    assert!(host_engine
        .handle_remote_packet(&move_pkt, &linux_injector, &linux_display)
        .is_ok());
    assert!(host_engine
        .handle_remote_packet(&click_pkt, &linux_injector, &linux_display)
        .is_ok());
    assert!(host_engine
        .handle_remote_packet(&key_pkt, &linux_injector, &linux_display)
        .is_ok());

    // 6. Retorno do cursor (ScreenLeave vindo do nó remoto)
    let leave_pkt = NexaPacket::ScreenLeave(ScreenLeave { timestamp_ms: 1000 });
    assert!(host_engine
        .handle_remote_packet(&leave_pkt, &linux_injector, &linux_display)
        .is_ok());

    // Host deve restaurar o controle local
    assert_eq!(host_engine.fsm().current_state(), &SessionState::LocalActive);
}

#[test]
fn test_mouse_coordinate_clamping_prevents_unbounded_drift() {
    let win_geo = ScreenGeometry::new(1920, 1080);
    let mut engine = SessionEngine::new("host", win_geo, 0);
    engine.topology_mut().register_screen("remote", ScreenGeometry::new(1920, 1080));
    engine.topology_mut().link_horizontal("host", "remote");

    // Ativa RemoteActive na borda
    let enter = engine.handle_local_mouse_move_abs(1919, 540).unwrap().unwrap();
    assert!(matches!(enter, NexaPacket::ScreenEnter(_)));
    assert!(matches!(engine.fsm().current_state(), SessionState::RemoteActive { .. }));

    // Simula rajada maciça de movimentos rápidos para a direita
    for _ in 0..100 {
        let _ = engine.handle_local_mouse_move_rel(500, 0);
    }

    // virtual_remote_x DEVE estar limitado a 1920.0 (largura da tela remota)
    assert!(
        engine.virtual_remote_x() <= 1920.0,
        "virtual_remote_x ({}) excedeu a largura da tela remota!",
        engine.virtual_remote_x()
    );
}

#[test]
fn test_queued_absolute_events_ignored_in_remote_mode() {
    let win_geo = ScreenGeometry::new(1920, 1080);
    let mut engine = SessionEngine::new("host", win_geo, 0);
    engine.topology_mut().register_screen("remote", ScreenGeometry::new(1920, 1080));
    engine.topology_mut().link_horizontal("host", "remote");

    // Ativa RemoteActive
    let _ = engine.handle_local_mouse_move_abs(1919, 540).unwrap();
    let initial_rx = engine.virtual_remote_x();

    // Eventos absolutos residuais (ex.: x=1919) enfileirados durante a transição
    let res = engine.handle_local_mouse_move_abs(1919, 540).unwrap();
    assert!(res.is_none(), "Evento absoluto não deve gerar pacote em RemoteActive");
    assert_eq!(
        engine.virtual_remote_x(),
        initial_rx,
        "Coordenada absoluta não pode ser somada como delta relativo!"
    );
}

#[test]
fn test_smooth_return_to_local_without_freeze() {
    let win_geo = ScreenGeometry::new(1920, 1080);
    let mut engine = SessionEngine::new("host", win_geo, 0);
    engine.topology_mut().register_screen("remote", ScreenGeometry::new(1920, 1080));
    engine.topology_mut().link_horizontal("host", "remote");

    // Ativa RemoteActive
    let _ = engine.handle_local_mouse_move_abs(1919, 540).unwrap();
    assert!(matches!(engine.fsm().current_state(), SessionState::RemoteActive { .. }));

    // Usuário move para a esquerda de volta à borda (virtual_remote_x inicia em 10.0)
    let leave = engine.handle_local_mouse_move_rel(-20, 0).unwrap();
    assert!(
        matches!(leave, Some(NexaPacket::ScreenLeave(_))),
        "Deve retornar ScreenLeave imediatamente ao tocar a borda esquerda"
    );
    assert_eq!(
        engine.fsm().current_state(),
        &SessionState::LocalActive,
        "Estado DEVE retornar para LocalActive sem travar ou necessitar de ESC"
    );
}

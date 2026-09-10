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

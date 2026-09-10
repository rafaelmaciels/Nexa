use bytes::BytesMut;
use nexa_core::{EdgeSide, ScreenGeometry, ScreenTopology, SessionFsm, SessionState};
use nexa_protocol::{
    decode_packet, encode_packet, MouseButton, MouseButtonMsg, MouseMotionMode, MouseMove,
    NexaPacket, ScreenEnter,
};

#[test]
fn test_full_session_and_packet_flow() {
    // 1. Configura a topologia
    let mut topology = ScreenTopology::new();
    topology.register_screen("win11", ScreenGeometry::new(1920, 1080));
    topology.register_screen("elem8", ScreenGeometry::new(1920, 1080));
    topology.link_horizontal("win11", "elem8");

    // 2. FSM no Windows com comutação imediata (delay zero para teste)
    let mut fsm = SessionFsm::new(0);
    assert_eq!(fsm.current_state(), &SessionState::LocalActive);

    // Cursor atinge a borda direita
    let edge_hit = fsm.on_edge_hit(EdgeSide::Right, "elem8");
    assert!(edge_hit);
    assert_eq!(
        fsm.current_state(),
        &SessionState::RemoteActive {
            active_target: "elem8".into()
        }
    );

    // Calcula a transição proporcional (saiu na altura 540)
    let (target, target_x, target_y) = topology
        .calculate_transition("win11", EdgeSide::Right, 1919, 540)
        .expect("Transição esperada");
    assert_eq!(target, "elem8");
    assert_eq!(target_x, 0);
    assert_eq!(target_y, 540);

    // 3. Serializa o pacote ScreenEnter de 50% de altura (Y normalizado = 32768)
    let enter_pkt = NexaPacket::ScreenEnter(ScreenEnter {
        normalized_x: 0,
        normalized_y: 32768,
        lock_keys_mask: 0,
    });
    let mut net_buffer = BytesMut::new();
    encode_packet(&enter_pkt, 1, &mut net_buffer);

    // 4. Decodifica no nó receptor
    let (seq, decoded) = decode_packet(&mut net_buffer)
        .expect("Decodificação sem erro")
        .expect("Pacote completo");
    assert_eq!(seq, 1);
    assert_eq!(decoded, enter_pkt);

    // 5. Transmissão de eventos de mouse e clique subsequentes
    let move_pkt = NexaPacket::MouseMove(MouseMove {
        mode: MouseMotionMode::Absolute,
        x: 100,
        y: 200,
    });
    encode_packet(&move_pkt, 2, &mut net_buffer);

    let click_pkt = NexaPacket::MouseButton(MouseButtonMsg {
        button: MouseButton::Left,
        is_down: true,
    });
    encode_packet(&click_pkt, 3, &mut net_buffer);

    // Decodifica primeiro o Move
    let (seq2, decoded_move) = decode_packet(&mut net_buffer).unwrap().unwrap();
    assert_eq!(seq2, 2);
    assert_eq!(decoded_move, move_pkt);

    // Decodifica em seguida o Click
    let (seq3, decoded_click) = decode_packet(&mut net_buffer).unwrap().unwrap();
    assert_eq!(seq3, 3);
    assert_eq!(decoded_click, click_pkt);

    // O buffer deve estar completamente consumido
    assert!(net_buffer.is_empty());
}

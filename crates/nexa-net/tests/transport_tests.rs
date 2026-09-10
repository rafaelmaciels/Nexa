use nexa_net::{HeartbeatTracker, NexaClient, NexaServer};
use nexa_protocol::{
    KeyEvent, KeyState, MouseMotionMode, MouseMove, NexaPacket, Ping,
};

#[tokio::test]
async fn test_tcp_handshake_sas_and_encrypted_exchange() {
    // 1. Inicia o servidor na porta 0 (porta efêmera livre do SO)
    let server = NexaServer::bind("127.0.0.1:0").await.expect("Bind do servidor falhou");
    let local_addr = server.local_addr().expect("Falha ao obter porta local");

    // 2. Task do Servidor
    let server_task = tokio::spawn(async move {
        let (mut conn, sas_pin) = server.accept_handshake().await.expect("Handshake no servidor falhou");

        // Recebe o MouseMove cifrado
        let packet1 = conn.recv_packet().await.expect("Erro ao receber pacote").expect("Fim prematuro");
        assert_eq!(
            packet1,
            NexaPacket::MouseMove(MouseMove {
                mode: MouseMotionMode::Absolute,
                x: 1920,
                y: 1080,
            })
        );

        // Recebe o KeyEvent cifrado
        let packet2 = conn.recv_packet().await.expect("Erro ao receber pacote").expect("Fim prematuro");
        assert_eq!(
            packet2,
            NexaPacket::KeyEvent(KeyEvent {
                scancode: 0x2A, // Left Shift
                state: KeyState::Down,
                modifiers: 1,
            })
        );

        // Envia um Ping cifrado
        let ping_pkt = NexaPacket::Ping(Ping { timestamp_nanos: 998877 });
        conn.send_packet(&ping_pkt).await.expect("Falha ao enviar Ping");

        // Recebe o Pong cifrado de volta
        let pong_pkt = conn.recv_packet().await.expect("Erro ao receber Pong").expect("Fim prematuro");
        assert!(matches!(pong_pkt, NexaPacket::Pong(_)));

        sas_pin
    });

    // 3. Conexão do Cliente
    let (mut client_conn, client_sas_pin) = NexaClient::connect(&local_addr.to_string())
        .await
        .expect("Conexão do cliente falhou");

    // 4. Cliente envia MouseMove cifrado
    client_conn
        .send_packet(&NexaPacket::MouseMove(MouseMove {
            mode: MouseMotionMode::Absolute,
            x: 1920,
            y: 1080,
        }))
        .await
        .expect("Falha ao enviar MouseMove");

    // 5. Cliente envia KeyEvent cifrado
    client_conn
        .send_packet(&NexaPacket::KeyEvent(KeyEvent {
            scancode: 0x2A,
            state: KeyState::Down,
            modifiers: 1,
        }))
        .await
        .expect("Falha ao enviar KeyEvent");

    // 6. Cliente recebe Ping e responde com Pong
    let ping_received = client_conn.recv_packet().await.expect("Erro no recv").expect("Fim prematuro");
    if let NexaPacket::Ping(ping) = ping_received {
        let pong = HeartbeatTracker::create_pong(&ping);
        client_conn.send_packet(&pong).await.expect("Falha ao enviar Pong");
    } else {
        panic!("Esperado pacote Ping");
    }

    // 7. Aguarda o servidor e compara o código SAS visual exibido
    let server_sas_pin = server_task.await.expect("Erro na task do servidor");

    // O código PIN SAS de 6 dígitos DEVE ser IDÊNTICO nas duas pontas para validação visual do usuário
    assert_eq!(client_sas_pin, server_sas_pin);
    assert_eq!(client_sas_pin.len(), 7); // ex: "482 731"
}

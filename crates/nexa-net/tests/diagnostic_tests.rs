use std::time::Duration;
use tokio::net::TcpListener;
use nexa_crypto::MachineIdentity;
use nexa_net::{run_connection_diagnostic, DiagnosticStepStatus};

#[tokio::test]
async fn test_diagnostic_connection_refused_scenario() {
    let port = 25899; // Porta de aplicação não alocada

    let identity = MachineIdentity::generate();
    let diag = run_connection_diagnostic(
        "127.0.0.1",
        port,
        &identity,
        &[],
        Duration::from_millis(1500),
    ).await;

    assert!(!diag.overall_success, "Diagnóstico não deve indicar sucesso em porta não escutada");
    assert!(!diag.possible_causes.is_empty(), "Diagnóstico deve listar possíveis causas");

    // Valida que o diagnóstico identificou falha de rede/porta ou timeout de firewall
    let identified_issue = matches!(diag.tcp_port_open, DiagnosticStepStatus::Failed(_))
        || matches!(diag.ip_connectivity, DiagnosticStepStatus::Failed(_));
    assert!(identified_issue, "Deveria ter diagnosticado falha de porta fechada ou firewall");

    // Deve sugerir causas reais (offline, firewall ou AP isolation)
    assert!(
        diag.possible_causes.iter().any(|c| c.contains("Firewall") || c.contains("offline") || c.contains("Nexa")),
        "Deveria orientar sobre causas prováveis no relatório"
    );
}

#[tokio::test]
async fn test_diagnostic_unroutable_timeout_scenario() {
    // Endereço IP não roteável (TEST-NET-1 reservado que sempre causa timeout)
    let non_routable_ip = "192.0.2.1"; // 192.0.2.0/24 TEST-NET-1 (RFC 5737)
    let identity = MachineIdentity::generate();

    let diag = run_connection_diagnostic(
        non_routable_ip,
        25800,
        &identity,
        &[],
        Duration::from_millis(300), // Timeout rápido
    ).await;

    // Conectividade IP deve falhar por Timeout
    assert!(matches!(diag.ip_connectivity, DiagnosticStepStatus::Failed(_)));
    let err_msg = match &diag.ip_connectivity {
        DiagnosticStepStatus::Failed(msg) => msg.clone(),
        _ => String::new(),
    };
    assert!(err_msg.contains("Timeout") || err_msg.contains("limite"));

    // Diagnóstico deve mencionar possíveis causas como isolamento de clientes (AP Isolation) ou offline
    assert!(diag.possible_causes.iter().any(|c| c.contains("offline") || c.contains("Isolamento")));
    assert!(!diag.overall_success);
}

#[tokio::test]
async fn test_diagnostic_server_listening_scenario() {
    // Inicia um listener TCP local simulando a porta aberta
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    // Spawn de tarefa para aceitar a conexão do teste
    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            use tokio::io::AsyncWriteExt;
            // Responde com o Magic "NEXA"
            let _ = socket.write_all(b"NEXA\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00").await;
        }
    });

    let identity = MachineIdentity::generate();
    let diag = run_connection_diagnostic(
        "127.0.0.1",
        port,
        &identity,
        &[],
        Duration::from_millis(1000),
    ).await;

    assert!(diag.ip_connectivity.is_success());
    assert!(diag.tcp_port_open.is_success());
    assert!(diag.nexa_service.is_success());
    assert!(diag.overall_success);
}

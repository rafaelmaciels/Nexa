use tempfile::tempdir;
use nexa_daemon::{DaemonOptions, NexaDaemonService};
use nexa_platform::LinuxAutostartManager;

#[cfg(windows)]
use nexa_platform::WindowsAutostartManager;

#[tokio::test]
async fn test_daemon_configuration_bootstrap() {
    let tmp = tempdir().expect("Falha ao criar tempdir");
    let config_path = tmp.path().join("test_nexa.toml");

    let options = DaemonOptions {
        config_path: config_path.clone(),
        is_server_mode: true,
        listen_port: 24805,
        enable_discovery: false, // Desativa broadcast durante o teste unitário
        connect_target: None,
    };

    let daemon = NexaDaemonService::new(options).expect("Falha ao instanciar daemon");

    // Verifica bootstrap da configuração
    assert!(config_path.exists(), "Arquivo nexa.toml deveria ter sido criado automaticamente");
    assert!(!daemon.device_name().is_empty());
    assert_eq!(daemon.public_key_hex().len(), 64, "Chave pública Ed25519 deve ter 64 caracteres hex");
}

#[tokio::test]
async fn test_daemon_lifecycle_start_and_stop() {
    let tmp = tempdir().expect("Falha ao criar tempdir");
    let config_path = tmp.path().join("lifecycle_nexa.toml");

    let options = DaemonOptions {
        config_path,
        is_server_mode: true,
        listen_port: 24806,
        enable_discovery: false,
        connect_target: None,
    };

    let daemon = NexaDaemonService::new(options).expect("Falha ao instanciar daemon");
    assert!(!daemon.is_running());

    daemon.start().await.expect("Falha ao iniciar daemon");
    assert!(daemon.is_running());

    daemon.stop();
    assert!(!daemon.is_running());
}

#[tokio::test]
async fn test_daemon_client_mode_initialization() {
    let tmp = tempdir().expect("Falha ao criar tempdir");
    let config_path = tmp.path().join("client_nexa.toml");

    let options = DaemonOptions {
        config_path,
        is_server_mode: false,
        listen_port: 24807,
        enable_discovery: false,
        connect_target: Some("127.0.0.1:24807".to_string()),
    };

    let daemon = NexaDaemonService::new(options).expect("Falha ao instanciar daemon em modo cliente");
    assert!(!daemon.is_running());
    daemon.start().await.expect("Falha ao iniciar daemon em modo cliente");
    assert!(daemon.is_running());
    daemon.stop();
    assert!(!daemon.is_running());
}

#[test]
fn test_autostart_generation_linux() {
    let service_content = LinuxAutostartManager::generate_systemd_user_service("/usr/bin/nexa-daemon");
    assert!(service_content.contains("[Unit]"));
    assert!(service_content.contains("ExecStart=/usr/bin/nexa-daemon --daemon"));
    assert!(service_content.contains("WantedBy=graphical-session.target"));

    let desktop_content = LinuxAutostartManager::generate_desktop_entry("/usr/bin/nexa-daemon");
    assert!(desktop_content.contains("[Desktop Entry]"));
    assert!(desktop_content.contains("Type=Application"));
    assert!(desktop_content.contains("Exec=/usr/bin/nexa-daemon --daemon"));
}

#[cfg(windows)]
#[test]
fn test_autostart_generation_windows() {
    let cmd = WindowsAutostartManager::generate_task_scheduler_command("C:\\Nexa\\nexa-daemon.exe");
    assert!(cmd.contains("schtasks /create /tn \"NexaDaemon\""));
    assert!(cmd.contains("/sc onlogon"));
    assert!(cmd.contains("/rl highest"));

    let reg = WindowsAutostartManager::generate_run_registry_command("C:\\Nexa\\nexa-daemon.exe");
    assert!(reg.contains("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run"));
    assert!(reg.contains("NexaDaemon"));
}

#[tokio::test]
async fn test_server_client_connection_and_no_freeze() {
    let tmp_server = tempdir().expect("tempdir server");
    let server_cfg = tmp_server.path().join("server_nexa.toml");

    let server_opts = DaemonOptions {
        config_path: server_cfg,
        is_server_mode: true,
        listen_port: 24810,
        enable_discovery: false,
        connect_target: None,
    };

    let server_daemon = NexaDaemonService::new(server_opts).expect("Servidor deve instanciar");
    server_daemon.start().await.expect("Servidor deve iniciar");
    assert!(server_daemon.is_running());

    // Aguarda bind do socket
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    let tmp_client = tempdir().expect("tempdir client");
    let client_cfg = tmp_client.path().join("client_nexa.toml");

    let client_opts = DaemonOptions {
        config_path: client_cfg,
        is_server_mode: false,
        listen_port: 24811,
        enable_discovery: false,
        connect_target: Some("127.0.0.1:24810".to_string()),
    };

    let client_daemon = NexaDaemonService::new(client_opts).expect("Cliente deve instanciar");
    client_daemon.start().await.expect("Cliente deve iniciar e conectar");
    assert!(client_daemon.is_running());

    // Aguarda handshake criptográfico e inicialização do capturador de periféricos no servidor
    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

    // Encerramento limpo de ambos os nós sem travamento
    client_daemon.stop();
    server_daemon.stop();
    assert!(!client_daemon.is_running());
    assert!(!server_daemon.is_running());
}

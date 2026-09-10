use std::env;
use std::path::PathBuf;
use std::process::exit;
use tracing::{error, info};
use nexa_common::{init_logger, LogLevel};
use nexa_daemon::{DaemonOptions, NexaDaemonService};

#[cfg(windows)]
use nexa_platform::WindowsAutostartManager;
use nexa_platform::LinuxAutostartManager;

const NEXA_VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        r#"
Nexa KVM Share Daemon v{}
Compartilhamento de mouse, teclado e clipboard de baixa latência e alta segurança.

USO:
    nexa-daemon [OPÇÕES]

OPÇÕES:
    -h, --help                     Exibe esta mensagem de ajuda
    -v, --version                  Exibe a versão instalada do Nexa
    -c, --config <ARQUIVO>         Caminho para arquivo de configuração TOML (Padrão: nexa.toml)
    -p, --port <PORTA>             Porta de escuta TCP para conexões de rede (Padrão: 25800)
    --server                       Executa em modo Servidor (Host que compartilha teclado e mouse)
    --client <IP[:PORTA]>          Executa em modo Cliente (Guest que recebe o controle do cursor)
    --connect <IP[:PORTA]>         Alias para --client
    --test-connection <IP[:PORTA]> Executa diagnóstico detalhado de rede e reachability
    --interfaces                   Lista interfaces de rede locais e seus endereços IP
    --no-discovery                 Desativa a descoberta automática de nós via UDP broadcast
    --autostart-enable             Gera e exibe as instruções de inicialização automática no SO
    --autostart-disable            Gera instruções para remoção da inicialização automática
    --daemon                       Executa silenciosamente em segundo plano
"#,
        NEXA_VERSION
    );
}

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    nexa_platform::enable_dpi_awareness();

    // 1. Inicializa o subsistema de logging estruturado
    init_logger(LogLevel::Info);

    let args: Vec<String> = env::args().collect();
    let mut options = DaemonOptions::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-v" | "--version" => {
                println!("nexa-daemon v{}", NEXA_VERSION);
                exit(0);
            }
            "-c" | "--config" => {
                if i + 1 < args.len() {
                    options.config_path = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    if let Ok(port) = args[i + 1].parse::<u16>() {
                        options.listen_port = port;
                    }
                    i += 1;
                }
            }
            "--server" => {
                options.is_server_mode = true;
            }
            "--client" | "--connect" => {
                options.is_server_mode = false;
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    options.connect_target = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--interfaces" => {
                println!("=== Interfaces de Rede Locais ===");
                for iface in nexa_net::detect_local_interfaces() {
                    let kind = if iface.is_loopback { "Loopback" } else { "Ativa" };
                    println!("• {:<30} [{}] : {}", iface.name, kind, iface.ip_address);
                }
                exit(0);
            }
            "--test-connection" => {
                if i + 1 < args.len() {
                    let target_arg = args[i + 1].clone();
                    let (ip, port) = if let Some((h, p)) = target_arg.split_once(':') {
                        (h.to_string(), p.parse::<u16>().unwrap_or(nexa_core::DEFAULT_PORT))
                    } else {
                        (target_arg, nexa_core::DEFAULT_PORT)
                    };

                    println!("Iniciando diagnóstico contra {}:{} ...", ip, port);
                    let identity = nexa_crypto::MachineIdentity::generate();
                    let diag = nexa_net::run_connection_diagnostic(
                        &ip,
                        port,
                        &identity,
                        &[],
                        std::time::Duration::from_millis(2000),
                    ).await;

                    println!("{}", nexa_ui::UiViewRenderer::render_diagnostic_panel(&diag));
                    exit(if diag.overall_success { 0 } else { 1 });
                } else {
                    eprintln!("Erro: --test-connection requer endereço IP (ex: 192.168.1.20 ou 192.168.1.20:25800)");
                    exit(1);
                }
            }
            "--no-discovery" => {
                options.enable_discovery = false;
            }
            "--autostart-enable" => {
                println!("=== Configuração de Inicialização Automática ===");
                #[cfg(windows)]
                {
                    let current_exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("nexa-daemon.exe"));
                    let exe_str = current_exe.to_string_lossy();
                    println!("\n[Windows 11 - Agendador de Tarefas (Recomendado para UAC)]:");
                    println!("{}", WindowsAutostartManager::generate_task_scheduler_command(&exe_str));
                    println!("\n[Windows 11 - Registro Run]:");
                    println!("{}", WindowsAutostartManager::generate_run_registry_command(&exe_str));
                }

                println!("\n[Linux elementary OS 8.1 - systemd user service (~/.config/systemd/user/nexa.service)]:");
                println!("{}", LinuxAutostartManager::generate_systemd_user_service("/usr/bin/nexa-daemon"));
                exit(0);
            }
            "--autostart-disable" => {
                #[cfg(windows)]
                {
                    println!("[Windows 11 - Remoção]:");
                    println!("{}", WindowsAutostartManager::generate_task_scheduler_delete_command());
                    println!("{}", WindowsAutostartManager::generate_run_registry_delete_command());
                }
                println!("[Linux - Remoção]: systemctl --user disable --now nexa.service");
                exit(0);
            }
            "--daemon" => {
                // Modo silencioso de background
            }
            unknown => {
                eprintln!("Opção desconhecida: {}", unknown);
                print_help();
                exit(1);
            }
        }
        i += 1;
    }

    info!("Iniciando Nexa Daemon v{}...", NEXA_VERSION);

    let daemon = match NexaDaemonService::new(options) {
        Ok(d) => d,
        Err(e) => {
            error!("Falha ao inicializar o daemon: {}", e);
            exit(1);
        }
    };

    if let Err(e) = daemon.start().await {
        error!("Erro durante a execução do daemon: {}", e);
        exit(1);
    }

    info!("Pressione Ctrl+C para encerrar o Nexa Daemon com segurança.");

    // Aguarda sinal de encerramento do sistema (SIGINT / Ctrl+C)
    if let Err(e) = tokio::signal::ctrl_c().await {
        error!("Erro ao aguardar sinal de encerramento: {}", e);
    }

    info!("Sinal de interrupção recebido!");
    daemon.stop();
    info!("Nexa Daemon encerrado com sucesso.");
}

/// Gerenciador de Inicialização Automática para Windows 11
pub struct WindowsAutostartManager;

impl WindowsAutostartManager {
    /// Gera comando para criar tarefa agendada de inicialização no logon com privilégios elevados
    pub fn generate_task_scheduler_command(exe_path: &str) -> String {
        format!(
            "schtasks /create /tn \"NexaDaemon\" /tr \"\\\"{}\\\" --daemon\" /sc onlogon /rl highest /f",
            exe_path
        )
    }

    /// Gera comando para remover tarefa agendada
    pub fn generate_task_scheduler_delete_command() -> String {
        "schtasks /delete /tn \"NexaDaemon\" /f".to_string()
    }

    /// Gera chave de registro do Windows Run
    pub fn generate_run_registry_command(exe_path: &str) -> String {
        format!(
            "reg add \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\" /v \"NexaDaemon\" /t REG_SZ /d \"\\\"{}\\\" --daemon\" /f",
            exe_path
        )
    }

    /// Gera comando para remover chave de registro do Windows Run
    pub fn generate_run_registry_delete_command() -> String {
        "reg delete \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\" /v \"NexaDaemon\" /f".to_string()
    }
}

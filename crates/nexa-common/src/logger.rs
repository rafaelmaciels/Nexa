use std::sync::Once;
use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();

/// Níveis de log suportados pelo Nexa de forma padronizada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    pub fn as_filter_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warning => "warn",
            LogLevel::Error => "error",
            LogLevel::Critical => "error",
        }
    }
}

/// Sanitiza strings para nunca expor dados confidenciais nos logs.
/// Remove ou mascara padrões como tokens, hashes de chave privada, senhas e códigos PIN.
pub fn sanitize_log_message(input: &str) -> String {
    let mut sanitized = input.to_string();

    // Mascara padrões de PIN numérico do pareamento (ex.: "482 731" ou "482731")
    let pin_regex = regex::Regex::new(r"(?i)(pin|code|pairing|sas)[:= ]+(\d{3}\s?\d{3})").unwrap();
    sanitized = pin_regex
        .replace_all(&sanitized, "$1: [REDACTED_PIN]")
        .to_string();

    // Mascara chaves privadas ou hexadecimais longos (ex.: chaves de 32 ou 64 bytes)
    let hex_key_regex =
        regex::Regex::new(r"(?i)(key|secret|private|token)[:= ]+([0-9a-fA-F]{32,128})").unwrap();
    sanitized = hex_key_regex
        .replace_all(&sanitized, "$1: [REDACTED_SECRET]")
        .to_string();

    // Mascara senhas
    let password_regex =
        regex::Regex::new(r"(?i)(password|passphrase)[:= ]+([^\s,;]+)").unwrap();
    sanitized = password_regex
        .replace_all(&sanitized, "$1: [REDACTED_PASSWORD]")
        .to_string();

    sanitized
}

/// Inicializa o subsistema de log com nível padrão.
pub fn init_logger(level: LogLevel) {
    INIT.call_once(|| {
        let filter = EnvFilter::new(format!("nexa={}", level.as_filter_str()));
        let subscriber = fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .compact()
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_pairing_pin() {
        let log = "Recebido pareamento com code: 482 731 da máquina remota";
        let sanitized = sanitize_log_message(log);
        assert!(!sanitized.contains("482 731"));
        assert!(sanitized.contains("[REDACTED_PIN]"));
    }

    #[test]
    fn test_sanitize_private_key() {
        let log = "Chave gerada secret: 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        let sanitized = sanitize_log_message(log);
        assert!(!sanitized.contains("9f86d081884c7d65"));
        assert!(sanitized.contains("[REDACTED_SECRET]"));
    }

    #[test]
    fn test_sanitize_password() {
        let log = "Tentativa de login com password: MinhaSenhaSuperSecreta123!";
        let sanitized = sanitize_log_message(log);
        assert!(!sanitized.contains("MinhaSenhaSuperSecreta123!"));
        assert!(sanitized.contains("[REDACTED_PASSWORD]"));
    }

    #[test]
    fn test_clean_log_untouched() {
        let log = "Cursor moveu para coordenadas x=1200, y=800 na tela 2";
        let sanitized = sanitize_log_message(log);
        assert_eq!(log, sanitized);
    }
}

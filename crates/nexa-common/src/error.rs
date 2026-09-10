use thiserror::Error;

/// Erros comuns transversais do ecossistema Nexa.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NexaError {
    #[error("Erro de I/O ou conexão: {0}")]
    Io(String),

    #[error("Erro de protocolo: {0}")]
    Protocol(String),

    #[error("Erro de autenticação ou segurança: {0}")]
    Security(String),

    #[error("Erro de configuração: {0}")]
    Config(String),

    #[error("Operação cancelada ou timeout: {0}")]
    Timeout(String),

    #[error("Erro de plataforma ou sistema operacional: {0}")]
    Platform(String),

    #[error("Erro interno do sistema: {0}")]
    Internal(String),
}

use tokio::net::TcpListener;
use nexa_common::NexaError;
use nexa_crypto::{EphemeralKeyPair, SecureChannel};
use crate::connection::SecureConnection;

/// Servidor TCP que escuta por novas conexões de clientes Nexa
pub struct NexaServer {
    listener: TcpListener,
}

impl NexaServer {
    pub async fn bind(addr: &str) -> Result<Self, NexaError> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao iniciar listener TCP em {}: {}", addr, e)))?;
        Ok(Self { listener })
    }

    pub fn local_addr(&self) -> Result<std::net::SocketAddr, NexaError> {
        self.listener
            .local_addr()
            .map_err(|e| NexaError::Io(format!("Falha ao obter endereço local: {}", e)))
    }

    /// Aguarda e aceita a próxima conexão de entrada, realizando o handshake de chaves efêmeras
    /// Retorna a `SecureConnection` criptografada e o código PIN SAS formatado para visualização
    pub async fn accept_handshake(&self) -> Result<(SecureConnection, String), NexaError> {
        let (socket, _peer_addr) = self
            .listener
            .accept()
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao aceitar conexão: {}", e)))?;

        let mut conn = SecureConnection::new(socket, None)?;

        // 1. Servidor gera par efêmero X25519
        let server_ephemeral = EphemeralKeyPair::generate();
        let server_pub = server_ephemeral.public_key_bytes();

        // 2. Envia chave pública efêmera ao cliente
        conn.send_raw(&server_pub).await?;

        // 3. Recebe chave pública efêmera do cliente
        let client_pub_bytes = conn.recv_raw().await?;
        if client_pub_bytes.len() != 32 {
            return Err(NexaError::Security("Chave efêmera do cliente com tamanho inválido".into()));
        }
        let mut client_pub = [0u8; 32];
        client_pub.copy_from_slice(&client_pub_bytes);

        // 4. Completa o Diffie-Hellman e deriva segredos e PIN SAS
        let (secrets, sas_pin) = server_ephemeral.complete_handshake(&client_pub)?;

        // 5. Inicializa o canal seguro
        let channel = SecureChannel::new(&secrets.encryption_key, true);
        conn.set_secure_channel(channel);

        Ok((conn, sas_pin))
    }
}

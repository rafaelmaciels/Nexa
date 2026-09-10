use tokio::net::TcpStream;
use nexa_common::NexaError;
use nexa_crypto::{EphemeralKeyPair, SecureChannel};
use crate::connection::SecureConnection;

/// Cliente TCP que conecta a um servidor Nexa e executa o handshake de pareamento
pub struct NexaClient;

impl NexaClient {
    /// Conecta ao servidor especificado e completa o handshake criptográfico X25519
    /// Retorna a `SecureConnection` criptografada e o código PIN SAS formatado para visualização
    pub async fn connect(server_addr: &str) -> Result<(SecureConnection, String), NexaError> {
        let socket = TcpStream::connect(server_addr)
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao conectar ao servidor em {}: {}", server_addr, e)))?;

        let mut conn = SecureConnection::new(socket, None)?;

        // 1. Recebe chave pública efêmera do servidor
        let server_pub_bytes = conn.recv_raw().await?;
        if server_pub_bytes.len() != 32 {
            return Err(NexaError::Security("Chave efêmera do servidor com tamanho inválido".into()));
        }
        let mut server_pub = [0u8; 32];
        server_pub.copy_from_slice(&server_pub_bytes);

        // 2. Cliente gera par efêmero X25519 e envia sua chave pública
        let client_ephemeral = EphemeralKeyPair::generate();
        let client_pub = client_ephemeral.public_key_bytes();
        conn.send_raw(&client_pub).await?;

        // 3. Completa o Diffie-Hellman e deriva segredos e PIN SAS
        let (secrets, sas_pin) = client_ephemeral.complete_handshake(&server_pub)?;

        // 4. Inicializa o canal seguro (lado cliente)
        let channel = SecureChannel::new(&secrets.encryption_key, false);
        conn.set_secure_channel(channel);

        Ok((conn, sas_pin))
    }
}

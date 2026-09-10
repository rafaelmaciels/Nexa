use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use bytes::BytesMut;
use nexa_common::NexaError;
use nexa_crypto::SecureChannel;
use nexa_protocol::{decode_packet, encode_packet, NexaPacket};

/// Conexão de rede segura e de baixa latência encapsulando um TcpStream
pub struct SecureConnection {
    stream: TcpStream,
    channel: Option<SecureChannel>,
    sequence_counter: u32,
}

impl SecureConnection {
    pub fn new(stream: TcpStream, channel: Option<SecureChannel>) -> Result<Self, NexaError> {
        stream
            .set_nodelay(true)
            .map_err(|e| NexaError::Io(format!("Falha ao ativar TCP_NODELAY: {}", e)))?;

        Ok(Self {
            stream,
            channel,
            sequence_counter: 1,
        })
    }

    /// Atribui o canal criptografado após a conclusão do handshake
    pub fn set_secure_channel(&mut self, channel: SecureChannel) {
        self.channel = Some(channel);
    }

    /// Envia dados brutos não criptografados com framing de tamanho (usado apenas no handshake inicial)
    pub async fn send_raw(&mut self, data: &[u8]) -> Result<(), NexaError> {
        let len = (data.len() as u32).to_le_bytes();
        self.stream
            .write_all(&len)
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao enviar tamanho bruto: {}", e)))?;
        self.stream
            .write_all(data)
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao enviar payload bruto: {}", e)))?;
        self.stream
            .flush()
            .await
            .map_err(|e| NexaError::Io(format!("Falha no flush do socket: {}", e)))?;
        Ok(())
    }

    /// Recebe dados brutos não criptografados (usado apenas no handshake inicial)
    pub async fn recv_raw(&mut self) -> Result<Vec<u8>, NexaError> {
        let mut len_buf = [0u8; 4];
        self.stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao ler tamanho bruto: {}", e)))?;

        let len = u32::from_le_bytes(len_buf) as usize;
        if len > 1024 * 1024 {
            return Err(NexaError::Protocol("Tamanho de mensagem bruta excede 1MB".into()));
        }

        let mut data = vec![0u8; len];
        self.stream
            .read_exact(&mut data)
            .await
            .map_err(|e| NexaError::Io(format!("Falha ao ler payload bruto: {}", e)))?;

        Ok(data)
    }

    /// Envia um NexaPacket devidamente encapsulado no protocolo binário e criptografado
    pub async fn send_packet(&mut self, packet: &NexaPacket) -> Result<(), NexaError> {
        // 1. Serializa no formato do protocolo binário (Header + Payload + CRC32)
        let mut encoded = BytesMut::new();
        encode_packet(packet, self.sequence_counter, &mut encoded);
        self.sequence_counter = self.sequence_counter.wrapping_add(1);

        // 2. Cifra o pacote através do SecureChannel (se ativado)
        let payload_to_send = if let Some(ref mut chan) = self.channel {
            chan.encrypt(&encoded)?
        } else {
            encoded.to_vec()
        };

        // 3. Envia no socket com prefixo de 4 bytes de tamanho
        self.send_raw(&payload_to_send).await
    }

    /// Recebe e decodifica o próximo NexaPacket disponível da conexão
    pub async fn recv_packet(&mut self) -> Result<Option<NexaPacket>, NexaError> {
        let raw_payload = match self.recv_raw().await {
            Ok(bytes) => bytes,
            Err(NexaError::Io(ref msg)) if msg.contains("unexpected end of file") || msg.contains("0 bytes read") => {
                return Ok(None); // Conexão fechada normalmente pelo par remoto
            }
            Err(e) => return Err(e),
        };

        // 1. Decifra se o canal seguro estiver ativo
        let decrypted_bytes = if let Some(ref mut chan) = self.channel {
            chan.decrypt(&raw_payload)?
        } else {
            raw_payload
        };

        // 2. Decodifica o pacote com verificação de CRC32
        let mut buf = BytesMut::from(&decrypted_bytes[..]);
        match decode_packet(&mut buf)? {
            Some((_seq, pkt)) => Ok(Some(pkt)),
            None => Err(NexaError::Protocol("Pacote incompleto recebido no quadro".into())),
        }
    }
}

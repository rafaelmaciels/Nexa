use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use nexa_common::NexaError;

pub const NONCE_SIZE: usize = 12;
pub const TAG_SIZE: usize = 16;

/// Canal seguro bidirecional com cifra autenticada ChaCha20-Poly1305
pub struct SecureChannel {
    cipher: ChaCha20Poly1305,
    send_nonce_counter: u64,
    last_recv_nonce_counter: u64,
    nonce_prefix: [u8; 4],
}

impl SecureChannel {
    /// Inicializa o canal com a chave simétrica de 32 bytes e um papel (Client vs Server)
    /// O papel diferencia o prefixo de nonce para que cliente e servidor nunca colidam nonces.
    pub fn new(key_bytes: &[u8; 32], is_server: bool) -> Self {
        let key = Key::from_slice(key_bytes);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce_prefix = if is_server {
            [0x53, 0x52, 0x56, 0x01] // "SRV\x01"
        } else {
            [0x43, 0x4C, 0x49, 0x01] // "CLI\x01"
        };

        Self {
            cipher,
            send_nonce_counter: 1,
            last_recv_nonce_counter: 0,
            nonce_prefix,
        }
    }

    /// Gera o próximo nonce de 12 bytes para envio
    fn next_send_nonce(&mut self) -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[0..4].copy_from_slice(&self.nonce_prefix);
        nonce[4..12].copy_from_slice(&self.send_nonce_counter.to_le_bytes());
        self.send_nonce_counter = self.send_nonce_counter.wrapping_add(1);
        nonce
    }

    /// Cifra os dados em texto plano e retorna `[Nonce (12B) | Ciphertext + Tag (16B)]`
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, NexaError> {
        let nonce_bytes = self.next_send_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| NexaError::Security(format!("Falha na cifragem do pacote: {}", e)))?;

        let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        Ok(output)
    }

    /// Decifra o quadro criptografado, validando o nonce (anti-replay) e o MAC Poly1305
    pub fn decrypt(&mut self, encrypted_frame: &[u8]) -> Result<Vec<u8>, NexaError> {
        if encrypted_frame.len() < NONCE_SIZE + TAG_SIZE {
            return Err(NexaError::Security("Quadro cifrado truncado ou inválido".into()));
        }

        let nonce_bytes = &encrypted_frame[0..NONCE_SIZE];
        let ciphertext = &encrypted_frame[NONCE_SIZE..];

        // Extrai o contador do nonce recebido
        let recv_counter = u64::from_le_bytes([
            nonce_bytes[4],
            nonce_bytes[5],
            nonce_bytes[6],
            nonce_bytes[7],
            nonce_bytes[8],
            nonce_bytes[9],
            nonce_bytes[10],
            nonce_bytes[11],
        ]);

        // Proteção contra Replay Attack: o contador deve ser estritamente crescente
        if recv_counter <= self.last_recv_nonce_counter {
            return Err(NexaError::Security(format!(
                "Ataque de Replay ou desordem detectado: contador {} <= {}",
                recv_counter, self.last_recv_nonce_counter
            )));
        }

        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| NexaError::Security("Falha de integridade Poly1305: payload adulterado".into()))?;

        // Atualiza o contador de nonce verificado
        self.last_recv_nonce_counter = recv_counter;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let mut server_channel = SecureChannel::new(&key, true);
        let mut client_channel = SecureChannel::new(&key, false);

        let message = b"Dados Confidenciais de MouseMove X=1920 Y=1080";

        // Servidor cifra
        let encrypted = server_channel.encrypt(message).expect("Falha ao cifrar");

        // O payload cifrado não contém a mensagem original
        assert!(!encrypted.windows(message.len()).any(|w| w == message));

        // Cliente decifra
        let decrypted = client_channel.decrypt(&encrypted).expect("Falha ao decifrar");
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_channel_tamper_detection() {
        let key = [0x77u8; 32];
        let mut server = SecureChannel::new(&key, true);
        let mut client = SecureChannel::new(&key, false);

        let message = b"Comando confidencial de clique";
        let mut encrypted = server.encrypt(message).unwrap();

        // Adultera 1 byte no ciphertext
        let last_idx = encrypted.len() - 1;
        encrypted[last_idx] ^= 0x01;

        let result = client.decrypt(&encrypted);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Poly1305"));
    }

    #[test]
    fn test_replay_attack_rejected() {
        let key = [0x99u8; 32];
        let mut server = SecureChannel::new(&key, true);
        let mut client = SecureChannel::new(&key, false);

        let message = b"Clique no Botao Esquerdo";
        let encrypted = server.encrypt(message).unwrap();

        // Primeira entrega: aceita
        let dec1 = client.decrypt(&encrypted);
        assert!(dec1.is_ok());

        // Segunda entrega (Replay Attack da mesma mensagem): REJEITA!
        let dec2 = client.decrypt(&encrypted);
        assert!(dec2.is_err());
        assert!(dec2.unwrap_err().to_string().contains("Replay"));
    }
}

use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use x25519_dalek::{EphemeralSecret, PublicKey};
use nexa_common::NexaError;

/// Par de chaves efêmero X25519 para handshake de pareamento e troca de chaves
pub struct EphemeralKeyPair {
    secret: EphemeralSecret,
    public_key: PublicKey,
}

impl EphemeralKeyPair {
    pub fn generate() -> Self {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);
        Self { secret, public_key }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public_key.as_bytes()
    }

    /// Executa o Diffie-Hellman com a chave pública efêmera recebida do peer
    /// e deriva a chave de sessão de 32 bytes e o PIN SAS de 6 dígitos formatado
    pub fn complete_handshake(
        self,
        peer_public_key_bytes: &[u8; 32],
    ) -> Result<(SessionSecrets, String), NexaError> {
        let peer_pub = PublicKey::from(*peer_public_key_bytes);
        let shared_secret = self.secret.diffie_hellman(&peer_pub);

        // Derivação com HKDF-SHA256
        let hk = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());

        // 1. Chave de sessão de 32 bytes para o ChaCha20-Poly1305
        let mut session_key = [0u8; 32];
        hk.expand(b"nexa-session-encryption-v1", &mut session_key)
            .map_err(|_| NexaError::Security("Falha na expansão HKDF da chave de sessão".into()))?;

        // 2. Derivação do PIN SAS numérico de 6 dígitos para confirmação visual
        let mut sas_bytes = [0u8; 4];
        hk.expand(b"nexa-sas-visual-confirmation-v1", &mut sas_bytes)
            .map_err(|_| NexaError::Security("Falha na derivação do PIN SAS".into()))?;

        let sas_num = (u32::from_be_bytes(sas_bytes) % 1_000_000) as u32;
        let sas_formatted = format!("{:03} {:03}", sas_num / 1000, sas_num % 1000);

        Ok((
            SessionSecrets {
                encryption_key: session_key,
            },
            sas_formatted,
        ))
    }
}

/// Segredos derivados da sessão
#[derive(Clone)]
pub struct SessionSecrets {
    pub encryption_key: [u8; 32],
}

/// Gera hash de confirmação do PIN para provar que a máquina aceitou o pareamento
pub fn compute_sas_confirmation(session_key: &[u8; 32], sas_pin: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(session_key);
    hasher.update(b":CONFIRM:");
    hasher.update(sas_pin.as_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x25519_handshake_agreement_and_sas_matching() {
        // Máquina A (ex: Windows 11)
        let party_a = EphemeralKeyPair::generate();
        let pub_a = party_a.public_key_bytes();

        // Máquina B (ex: elementary OS 8.1)
        let party_b = EphemeralKeyPair::generate();
        let pub_b = party_b.public_key_bytes();

        // A recebe a chave pública de B
        let (secrets_a, pin_a) = party_a.complete_handshake(&pub_b).unwrap();

        // B recebe a chave pública de A
        let (secrets_b, pin_b) = party_b.complete_handshake(&pub_a).unwrap();

        // O segredo derivado deve ser estritamente IDÊNTICO
        assert_eq!(secrets_a.encryption_key, secrets_b.encryption_key);

        // O código numérico SAS de 6 dígitos DEVE ser IDÊNTICO nas duas telas!
        assert_eq!(pin_a, pin_b);
        assert_eq!(pin_a.len(), 7); // formato "123 456" tem 7 caracteres

        // Hash de confirmação SAS
        let confirm_a = compute_sas_confirmation(&secrets_a.encryption_key, &pin_a);
        let confirm_b = compute_sas_confirmation(&secrets_b.encryption_key, &pin_b);
        assert_eq!(confirm_a, confirm_b);
    }
}

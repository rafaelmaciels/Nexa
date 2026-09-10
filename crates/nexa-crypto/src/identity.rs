use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use nexa_common::NexaError;

/// Identidade estável de longo prazo de uma máquina no ecossistema Nexa.
pub struct MachineIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl MachineIdentity {
    /// Gera um novo par de chaves seguro utilizando o CSPRNG do sistema operacional
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Carrega a partir de 32 bytes de segredo
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Retorna os bytes da chave pública (32 bytes)
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Retorna a chave pública em formato hexadecimal legível
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key_bytes())
    }

    /// Assina uma mensagem com a chave privada da máquina
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let sig: Signature = self.signing_key.sign(message);
        sig.to_bytes()
    }

    /// Valida uma assinatura com uma chave pública remota
    pub fn verify_peer(
        peer_public_key_bytes: &[u8; 32],
        message: &[u8],
        signature_bytes: &[u8; 64],
    ) -> Result<(), NexaError> {
        let verifying_key = VerifyingKey::from_bytes(peer_public_key_bytes)
            .map_err(|e| NexaError::Security(format!("Chave pública do peer inválida: {}", e)))?;
        let signature = Signature::from_bytes(signature_bytes);
        verifying_key
            .verify(message, &signature)
            .map_err(|e| NexaError::Security(format!("Assinatura do peer inválida: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_sign_and_verify() {
        let host_a = MachineIdentity::generate();
        let message = b"Nexa-Handshake-Auth-Test";

        let signature = host_a.sign(message);
        let pub_key = host_a.public_key_bytes();

        let result = MachineIdentity::verify_peer(&pub_key, message, &signature);
        assert!(result.is_ok());

        // Teste de adulteração na mensagem
        let corrupted_message = b"Nexa-Handshake-TAMPERED";
        let bad_result = MachineIdentity::verify_peer(&pub_key, corrupted_message, &signature);
        assert!(bad_result.is_err());
    }

    #[test]
    fn test_public_key_hex_length() {
        let identity = MachineIdentity::generate();
        let hex = identity.public_key_hex();
        assert_eq!(hex.len(), 64); // 32 bytes = 64 caracteres hex
    }
}

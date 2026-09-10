use sha2::{Digest, Sha256};
use tracing::{debug, warn};
use nexa_common::NexaError;
use nexa_platform::ClipboardManager;
use nexa_protocol::{ClipboardData, NexaPacket};

/// Limite padrão de payload de clipboard: 10 Megabytes
pub const DEFAULT_MAX_CLIPBOARD_BYTES: usize = 10 * 1024 * 1024;

/// Motor de Sincronização de Área de Transferência com prevenção ativa contra loops infinitos (Anti-Echo)
pub struct ClipboardSyncEngine {
    last_sent_hash: Option<[u8; 32]>,
    last_received_hash: Option<[u8; 32]>,
    max_payload_bytes: usize,
}

impl Default for ClipboardSyncEngine {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_CLIPBOARD_BYTES)
    }
}

impl ClipboardSyncEngine {
    pub fn new(max_payload_bytes: usize) -> Self {
        Self {
            last_sent_hash: None,
            last_received_hash: None,
            max_payload_bytes,
        }
    }

    /// Calcula o hash SHA-256 dos dados para identificação inequívoca de conteúdo
    pub fn compute_hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    pub fn last_sent_hash(&self) -> Option<[u8; 32]> {
        self.last_sent_hash
    }

    pub fn last_received_hash(&self) -> Option<[u8; 32]> {
        self.last_received_hash
    }

    pub fn set_max_payload_bytes(&mut self, max: usize) {
        self.max_payload_bytes = max;
    }

    /// Processa uma cópia detectada no sistema operacional local
    /// Retorna `Some(NexaPacket::Clipboard)` para envio à rede, ou `None` caso seja suprimido (eco ou duplicata)
    pub fn on_local_clipboard_changed(&mut self, text: &str) -> Option<NexaPacket> {
        let bytes = text.as_bytes();

        if bytes.len() > self.max_payload_bytes {
            warn!(
                "Conteúdo do clipboard local ({} bytes) excede o limite máximo permitido ({} bytes); transmissão cancelada.",
                bytes.len(),
                self.max_payload_bytes
            );
            return None;
        }

        let hash = Self::compute_hash(bytes);

        // Prevenção de loop infinito (Anti-Echo):
        // Se este hash for idêntico ao último que recebemos da rede, esta alteração foi disparada
        // pela nossa própria injeção remota. Reenviar causaria um loop sem fim.
        if Some(hash) == self.last_received_hash {
            debug!("Clipboard local corresponde ao último pacote remoto recebido (eco suprimido com sucesso).");
            return None;
        }

        // Idempotência: não reenviar conteúdo repetido
        if Some(hash) == self.last_sent_hash {
            debug!("Clipboard local já foi transmitido anteriormente (duplicata suprimida).");
            return None;
        }

        debug!(
            "Novo conteúdo de clipboard local detectado ({} bytes). Preparando transmissão remota.",
            bytes.len()
        );
        self.last_sent_hash = Some(hash);

        Some(NexaPacket::Clipboard(ClipboardData::new_utf8(text)))
    }

    /// Processa dados de clipboard recebidos de um nó remoto e aplica no clipboard do SO local
    /// Retorna `Ok(true)` se o clipboard local foi atualizado, ou `Ok(false)` se foi descartado/idempotente
    pub fn on_remote_clipboard_received<C: ClipboardManager>(
        &mut self,
        text: &str,
        clip_mgr: &C,
    ) -> Result<bool, NexaError> {
        let bytes = text.as_bytes();

        if bytes.len() > self.max_payload_bytes {
            warn!(
                "Payload de clipboard remoto ({} bytes) excede o limite permitido ({} bytes). Descartando.",
                bytes.len(),
                self.max_payload_bytes
            );
            return Ok(false);
        }

        let hash = Self::compute_hash(bytes);

        // Se já possuímos exatamente esse conteúdo como último enviado ou recebido, não precisamos reinjetar
        if Some(hash) == self.last_sent_hash || Some(hash) == self.last_received_hash {
            debug!("Clipboard remoto descartado (conteúdo já presente localmente).");
            return Ok(false);
        }

        // Registra o hash ANTES de injetar no clipboard local, para que qualquer evento imediato
        // gerado pelo SO seja prontamente reconhecido como eco e ignorado
        self.last_received_hash = Some(hash);

        debug!(
            "Injetando clipboard remoto no sistema operacional local ({} bytes).",
            bytes.len()
        );
        clip_mgr.set_text(text)?;

        Ok(true)
    }

    /// Disparado quando o mouse cruza a fronteira física da tela (ScreenEnter)
    /// Lê a área de transferência local e, se houver alteração não sincronizada, retorna o pacote para envio imediato
    pub fn sync_on_screen_switch<C: ClipboardManager>(
        &mut self,
        clip_mgr: &C,
    ) -> Result<Option<NexaPacket>, NexaError> {
        if let Some(text) = clip_mgr.get_text()? {
            Ok(self.on_local_clipboard_changed(&text))
        } else {
            Ok(None)
        }
    }
}

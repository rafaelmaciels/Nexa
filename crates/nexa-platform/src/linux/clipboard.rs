use std::sync::Mutex;
use tracing::debug;
use nexa_common::NexaError;
use crate::traits::ClipboardManager;

/// Gerenciador de Área de Transferência para Linux (elementary OS 8.1 / Wayland)
pub struct LinuxClipboardManager {
    // Armazenamento em memória e buffer para integração Wayland Portal
    content: Mutex<Option<String>>,
}

impl Default for LinuxClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxClipboardManager {
    pub fn new() -> Self {
        Self {
            content: Mutex::new(None),
        }
    }

    /// Simula recebimento de nova seleção vinda do compositor Wayland Gala via Portal
    pub fn on_wayland_selection_changed(&self, text: Option<String>) {
        if let Ok(mut lock) = self.content.lock() {
            *lock = text;
        }
    }
}

impl ClipboardManager for LinuxClipboardManager {
    fn get_text(&self) -> Result<Option<String>, NexaError> {
        let lock = self.content.lock().map_err(|_| {
            NexaError::Platform("Falha ao adquirir lock do clipboard Linux".into())
        })?;
        debug!("LinuxClipboardManager::get_text: {:?}", lock.as_deref());
        Ok(lock.clone())
    }

    fn set_text(&self, text: &str) -> Result<(), NexaError> {
        let mut lock = self.content.lock().map_err(|_| {
            NexaError::Platform("Falha ao adquirir lock do clipboard Linux".into())
        })?;
        debug!("LinuxClipboardManager::set_text: {}", text);
        *lock = Some(text.to_string());
        Ok(())
    }
}

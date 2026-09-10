use std::sync::Mutex;
use std::time::Duration;
use tracing::{debug, warn};
use nexa_common::NexaError;
use crate::traits::ClipboardManager;

#[cfg(windows)]
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
};

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GlobalFree(hmem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

const CF_UNICODETEXT: u32 = 13;

/// Gerenciador de Área de Transferência nativo para Windows 11
pub struct WindowsClipboardManager {
    // Fallback de memória para sessões headless ou de testes automatizados
    in_memory_fallback: Mutex<Option<String>>,
}

impl Default for WindowsClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsClipboardManager {
    pub fn new() -> Self {
        Self {
            in_memory_fallback: Mutex::new(None),
        }
    }

    /// Tenta abrir o clipboard do Windows com retries em caso de contenção temporária
    #[cfg(windows)]
    fn try_open_clipboard(&self) -> bool {
        for attempt in 0..5 {
            if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
                return true;
            }
            if attempt < 4 {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        false
    }
}

impl ClipboardManager for WindowsClipboardManager {
    fn get_text(&self) -> Result<Option<String>, NexaError> {
        #[cfg(windows)]
        {
            if !self.try_open_clipboard() {
                debug!("Não foi possível abrir o clipboard Win32 (possível sessão headless); usando fallback");
                let guard = self.in_memory_fallback.lock().map_err(|_| {
                    NexaError::Platform("Falha ao adquirir lock do clipboard fallback".into())
                })?;
                return Ok(guard.clone());
            }

            // Guard RAII para fechar o clipboard com segurança
            struct ClipboardGuard;
            impl Drop for ClipboardGuard {
                fn drop(&mut self) {
                    unsafe {
                        CloseClipboard();
                    }
                }
            }
            let _guard = ClipboardGuard;

            let handle = unsafe { GetClipboardData(CF_UNICODETEXT) };
            if handle.is_null() {
                // Sem texto unicode no clipboard
                return Ok(None);
            }

            let ptr = unsafe { GlobalLock(handle) } as *const u16;
            if ptr.is_null() {
                return Ok(None);
            }

            // Lê a string UTF-16 terminada em nulo
            let mut len = 0;
            unsafe {
                while *ptr.add(len) != 0 {
                    len += 1;
                }
            }

            let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
            let result = String::from_utf16_lossy(slice);

            unsafe {
                GlobalUnlock(handle);
            }

            Ok(Some(result))
        }

        #[cfg(not(windows))]
        {
            let guard = self.in_memory_fallback.lock().map_err(|_| {
                NexaError::Platform("Falha ao adquirir lock do clipboard fallback".into())
            })?;
            Ok(guard.clone())
        }
    }

    fn set_text(&self, text: &str) -> Result<(), NexaError> {
        // Atualiza o fallback em memória
        if let Ok(mut guard) = self.in_memory_fallback.lock() {
            *guard = Some(text.to_string());
        }

        #[cfg(windows)]
        {
            let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes_len = utf16.len() * std::mem::size_of::<u16>();

            let h_mem = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes_len) };
            if h_mem.is_null() {
                return Err(NexaError::Platform("Falha ao alocar memória global para clipboard".into()));
            }

            let ptr = unsafe { GlobalLock(h_mem) } as *mut u16;
            if ptr.is_null() {
                unsafe { GlobalFree(h_mem) };
                return Err(NexaError::Platform("Falha ao bloquear memória para clipboard".into()));
            }

            unsafe {
                std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
                GlobalUnlock(h_mem);
            }

            if !self.try_open_clipboard() {
                warn!("Não foi possível abrir o clipboard Win32 para escrita; gravado em fallback");
                unsafe { GlobalFree(h_mem) };
                return Ok(());
            }

            unsafe {
                EmptyClipboard();
                let set_res = SetClipboardData(CF_UNICODETEXT, h_mem);
                CloseClipboard();

                if set_res.is_null() {
                    GlobalFree(h_mem);
                    return Err(NexaError::Platform("Falha ao executar SetClipboardData".into()));
                }
            }

            Ok(())
        }

        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
}

use nexa_common::NexaError;
use crate::traits::ScreenManager;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{POINT, RECT};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ClipCursor, GetCursorPos, GetSystemMetrics, SetCursorPos, SetProcessDPIAware,
    SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

/// Ativa percepção de DPI (Per-Monitor DPI Aware) para evitar distorção de coordenadas
#[cfg(windows)]
pub fn enable_dpi_awareness() {
    unsafe {
        let _ = SetProcessDPIAware();
    }
}

#[cfg(not(windows))]
pub fn enable_dpi_awareness() {}

/// Gerenciador de monitores e cursor nativo para Windows
#[derive(Default)]
pub struct WindowsDisplayManager;

impl WindowsDisplayManager {
    pub fn new() -> Self {
        enable_dpi_awareness();
        Self
    }
}

impl ScreenManager for WindowsDisplayManager {
    fn get_screen_bounds(&self) -> Result<(i32, i32, i32, i32), NexaError> {
        #[cfg(windows)]
        unsafe {
            let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

            if width <= 0 || height <= 0 {
                return Err(NexaError::Internal(
                    "Falha ao consultar dimensões de tela virtual do Windows".into(),
                ));
            }

            Ok((x, y, width, height))
        }
        #[cfg(not(windows))]
        {
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }

    fn get_cursor_position(&self) -> Result<(i32, i32), NexaError> {
        #[cfg(windows)]
        unsafe {
            let mut pt = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut pt) != 0 {
                return Ok((pt.x, pt.y));
            }
            // Em sessões headless ou processos de background do Windows sem desktop interativo,
            // utiliza o centro da tela virtual como posição padrão
            let (vx, vy, vw, vh) = self.get_screen_bounds()?;
            Ok((vx + vw / 2, vy + vh / 2))
        }
        #[cfg(not(windows))]
        {
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }

    fn set_cursor_position(&self, x: i32, y: i32) -> Result<(), NexaError> {
        #[cfg(windows)]
        unsafe {
            let _ = SetCursorPos(x, y);
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (x, y);
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }

    fn clip_cursor(&self, rect: (i32, i32, i32, i32)) -> Result<(), NexaError> {
        #[cfg(windows)]
        unsafe {
            let win_rect = RECT {
                left: rect.0,
                top: rect.1,
                right: rect.2,
                bottom: rect.3,
            };
            if ClipCursor(&win_rect) == 0 {
                return Err(NexaError::Internal("ClipCursor falhou".into()));
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = rect;
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }

    fn unclip_cursor(&self) -> Result<(), NexaError> {
        #[cfg(windows)]
        unsafe {
            // Passar null libera o confinamento no Windows
            ClipCursor(std::ptr::null());
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_virtual_screen_geometry() {
        let dm = WindowsDisplayManager::new();
        let (x, y, width, height) = dm.get_screen_bounds().expect("Deve obter resolução do SO");
        assert!(width > 0);
        assert!(height > 0);
        println!("Tela virtual detectada: X={}, Y={}, W={}, H={}", x, y, width, height);

        let (cx, cy) = dm.get_cursor_position().expect("Deve obter cursor");
        println!("Posição atual do cursor: {}, {}", cx, cy);
    }
}

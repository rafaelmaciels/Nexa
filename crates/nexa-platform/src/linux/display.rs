use nexa_common::NexaError;
use crate::traits::ScreenManager;

/// Gerenciador de monitores e geometria para Linux (Wayland / Gala compositor)
pub struct LinuxDisplayManager {
    cached_bounds: (i32, i32, i32, i32),
}

impl Default for LinuxDisplayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxDisplayManager {
    pub fn new() -> Self {
        // Por padrão inicia com resolução desktop padrão até consulta ao compositor
        Self {
            cached_bounds: (0, 0, 1920, 1080),
        }
    }

    pub fn set_dimensions(&mut self, width: i32, height: i32) {
        self.cached_bounds.2 = width;
        self.cached_bounds.3 = height;
    }
}

impl ScreenManager for LinuxDisplayManager {
    fn get_screen_bounds(&self) -> Result<(i32, i32, i32, i32), NexaError> {
        Ok(self.cached_bounds)
    }

    fn get_cursor_position(&self) -> Result<(i32, i32), NexaError> {
        // No Wayland, a posição global do cursor é gerenciada pelo compositor e portal
        let (x, y, w, h) = self.cached_bounds;
        Ok((x + w / 2, y + h / 2))
    }

    fn set_cursor_position(&self, x: i32, y: i32) -> Result<(), NexaError> {
        let _ = (x, y);
        Ok(())
    }

    fn clip_cursor(&self, rect: (i32, i32, i32, i32)) -> Result<(), NexaError> {
        let _ = rect;
        // No Wayland, o confinamento é realizado pela ativação da barreira no portal InputCapture
        Ok(())
    }

    fn unclip_cursor(&self) -> Result<(), NexaError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_display_bounds() {
        let mut dm = LinuxDisplayManager::new();
        dm.set_dimensions(2560, 1440);

        let (_x, _y, w, h) = dm.get_screen_bounds().unwrap();
        assert_eq!(w, 2560);
        assert_eq!(h, 1440);

        let (cx, cy) = dm.get_cursor_position().unwrap();
        assert_eq!(cx, 1280);
        assert_eq!(cy, 720);
    }
}

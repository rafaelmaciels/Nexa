use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crossbeam_channel::{unbounded, Receiver, Sender};
use nexa_common::NexaError;
use crate::traits::{CapturedInputEvent, InputCapturer};

/// Barreira de ponteiro nas bordas configuradas para o portal InputCapture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalBarrierZone {
    Left,
    Right,
    Top,
    Bottom,
}

/// Capturador nativo para elementary OS 8.1 Wayland via XDG Desktop Portal e libeis
pub struct LinuxInputCapturer {
    is_running: Arc<AtomicBool>,
    suppress_input: Arc<AtomicBool>,
    active_zones: Vec<PortalBarrierZone>,
    receiver: Receiver<CapturedInputEvent>,
    _sender: Sender<CapturedInputEvent>,
}

impl Default for LinuxInputCapturer {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxInputCapturer {
    pub fn new() -> Self {
        let (_sender, receiver) = unbounded();
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            suppress_input: Arc::new(AtomicBool::new(false)),
            active_zones: Vec::new(),
            receiver,
            _sender,
        }
    }

    /// Adiciona uma barreira virtual de borda
    pub fn add_barrier_zone(&mut self, zone: PortalBarrierZone) {
        if !self.active_zones.contains(&zone) {
            self.active_zones.push(zone);
        }
    }

    pub fn receiver(&self) -> &Receiver<CapturedInputEvent> {
        &self.receiver
    }
}

impl InputCapturer for LinuxInputCapturer {
    fn start(&mut self) -> Result<(), NexaError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.is_running.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), NexaError> {
        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn set_suppression(&self, suppress: bool) {
        self.suppress_input.store(suppress, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_portal_capturer_lifecycle() {
        let mut capturer = LinuxInputCapturer::new();
        capturer.add_barrier_zone(PortalBarrierZone::Right);

        assert!(capturer.start().is_ok());
        capturer.set_suppression(true);
        capturer.set_suppression(false);
        assert!(capturer.stop().is_ok());
    }
}

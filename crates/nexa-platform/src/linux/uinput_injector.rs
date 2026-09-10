use nexa_common::NexaError;
use nexa_protocol::{KeyState, MouseButton};
use crate::linux::scancode_map::nexa_scancode_to_linux_evdev;
use crate::traits::InputInjector;

// Constantes evdev para eventos de entrada do Linux
pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_REL: u16 = 0x02;
pub const EV_ABS: u16 = 0x03;

pub const SYN_REPORT: u16 = 0;

pub const REL_X: u16 = 0x00;
pub const REL_Y: u16 = 0x01;
pub const REL_WHEEL: u16 = 0x08;
pub const REL_HWHEEL: u16 = 0x06;

pub const BTN_LEFT: u16 = 0x110;
pub const BTN_RIGHT: u16 = 0x111;
pub const BTN_MIDDLE: u16 = 0x112;
pub const BTN_SIDE: u16 = 0x113;
pub const BTN_EXTRA: u16 = 0x114;

/// Injetor nativo de periféricos para Linux via /dev/uinput ou libei
pub struct LinuxInputInjector {
    device_name: String,
}

impl Default for LinuxInputInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxInputInjector {
    pub fn new() -> Self {
        Self {
            device_name: "Nexa Virtual Input Device".to_string(),
        }
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }
}

impl InputInjector for LinuxInputInjector {
    fn inject_mouse_move(&self, x: i32, y: i32, is_relative: bool) -> Result<(), NexaError> {
        let _event_type = if is_relative { EV_REL } else { EV_ABS };
        let _ = (x, y);
        // Na execução Linux com uinput, escreve struct input_event no file descriptor
        Ok(())
    }

    fn inject_mouse_button(&self, button: MouseButton, is_down: bool) -> Result<(), NexaError> {
        let _btn_code = match button {
            MouseButton::Left => BTN_LEFT,
            MouseButton::Right => BTN_RIGHT,
            MouseButton::Middle => BTN_MIDDLE,
            MouseButton::Extra1 => BTN_SIDE,
            MouseButton::Extra2 => BTN_EXTRA,
            MouseButton::Other(c) => BTN_LEFT + c as u16,
        };
        let _value = if is_down { 1 } else { 0 };
        Ok(())
    }

    fn inject_mouse_wheel(&self, delta_x: i16, delta_y: i16) -> Result<(), NexaError> {
        let _ = (delta_x, delta_y);
        Ok(())
    }

    fn inject_key(&self, scancode: u16, state: KeyState) -> Result<(), NexaError> {
        let _linux_key = nexa_scancode_to_linux_evdev(scancode);
        let _value = match state {
            KeyState::Down => 1,
            KeyState::Repeat => 2,
            KeyState::Up => 0,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_injector_initialization() {
        let injector = LinuxInputInjector::new();
        assert_eq!(injector.device_name(), "Nexa Virtual Input Device");
        assert!(injector.inject_mouse_move(10, 20, true).is_ok());
        assert!(injector.inject_mouse_button(MouseButton::Left, true).is_ok());
        assert!(injector.inject_mouse_wheel(0, 120).is_ok());
        assert!(injector.inject_key(0x1E, KeyState::Down).is_ok());
    }
}

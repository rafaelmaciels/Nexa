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

#[cfg(target_os = "linux")]
const UI_SET_EVBIT: libc::c_ulong = 0x40045564;
#[cfg(target_os = "linux")]
const UI_SET_KEYBIT: libc::c_ulong = 0x40045565;
#[cfg(target_os = "linux")]
const UI_SET_RELBIT: libc::c_ulong = 0x40045566;
#[cfg(target_os = "linux")]
const UI_DEV_CREATE: libc::c_ulong = 0x5501;
#[cfg(target_os = "linux")]
const UI_DEV_DESTROY: libc::c_ulong = 0x5502;

#[cfg(target_os = "linux")]
#[repr(C)]
struct UinputSetup {
    id: libc::input_id,
    name: [libc::c_char; 80],
    ff_effects_max: libc::c_uint,
}

#[cfg(target_os = "linux")]
const UI_DEV_SETUP: libc::c_ulong = 0x405c5503;

/// Injetor nativo de periféricos para Linux via /dev/uinput
pub struct LinuxInputInjector {
    device_name: String,
    #[cfg(target_os = "linux")]
    fd: Option<std::sync::Mutex<std::fs::File>>,
}

impl Default for LinuxInputInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxInputInjector {
    pub fn new() -> Self {
        let device_name = "Nexa Virtual Input Device".to_string();

        #[cfg(target_os = "linux")]
        {
            use std::fs::OpenOptions;
            use std::os::unix::fs::OpenOptionsExt;
            use std::os::unix::io::AsRawFd;

            let file_res = OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_NONBLOCK)
                .open("/dev/uinput")
                .or_else(|_| {
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .custom_flags(libc::O_NONBLOCK)
                        .open("/dev/input/uinput")
                });

            let fd_opt = match file_res {
                Ok(file) => {
                    let raw = file.as_raw_fd();
                    unsafe {
                        libc::ioctl(raw, UI_SET_EVBIT, EV_SYN as libc::c_int);
                        libc::ioctl(raw, UI_SET_EVBIT, EV_KEY as libc::c_int);
                        libc::ioctl(raw, UI_SET_EVBIT, EV_REL as libc::c_int);

                        libc::ioctl(raw, UI_SET_RELBIT, REL_X as libc::c_int);
                        libc::ioctl(raw, UI_SET_RELBIT, REL_Y as libc::c_int);
                        libc::ioctl(raw, UI_SET_RELBIT, REL_WHEEL as libc::c_int);
                        libc::ioctl(raw, UI_SET_RELBIT, REL_HWHEEL as libc::c_int);

                        libc::ioctl(raw, UI_SET_KEYBIT, BTN_LEFT as libc::c_int);
                        libc::ioctl(raw, UI_SET_KEYBIT, BTN_RIGHT as libc::c_int);
                        libc::ioctl(raw, UI_SET_KEYBIT, BTN_MIDDLE as libc::c_int);
                        libc::ioctl(raw, UI_SET_KEYBIT, BTN_SIDE as libc::c_int);
                        libc::ioctl(raw, UI_SET_KEYBIT, BTN_EXTRA as libc::c_int);

                        for key in 1..=255 {
                            libc::ioctl(raw, UI_SET_KEYBIT, key as libc::c_int);
                        }

                        let mut setup: UinputSetup = std::mem::zeroed();
                        let name_bytes = b"Nexa Virtual Input Device\0";
                        for (i, &b) in name_bytes.iter().enumerate().take(setup.name.len() - 1) {
                            setup.name[i] = b as libc::c_char;
                        }
                        setup.id.bustype = libc::BUS_USB;
                        setup.id.vendor = 0x1234;
                        setup.id.product = 0x5678;
                        setup.id.version = 1;

                        if libc::ioctl(raw, UI_DEV_SETUP, &setup) < 0 {
                            let mut uidev: libc::uinput_user_dev = std::mem::zeroed();
                            for (i, &b) in name_bytes.iter().enumerate().take(uidev.name.len() - 1) {
                                uidev.name[i] = b as libc::c_char;
                            }
                            uidev.id.bustype = libc::BUS_USB;
                            uidev.id.vendor = 0x1234;
                            uidev.id.product = 0x5678;
                            uidev.id.version = 1;
                            libc::write(raw, &uidev as *const _ as *const libc::c_void, std::mem::size_of::<libc::uinput_user_dev>());
                        }

                        if libc::ioctl(raw, UI_DEV_CREATE) < 0 {
                            tracing::warn!("Falha ao executar UI_DEV_CREATE no uinput");
                            None
                        } else {
                            tracing::info!("Dispositivo virtual de entrada Nexa criado com sucesso em /dev/uinput!");
                            Some(std::sync::Mutex::new(file))
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Não foi possível abrir /dev/uinput ({}). Se estiver no Linux, certifique-se de pertencer ao grupo input: sudo usermod -aG input $USER", e);
                    None
                }
            };

            Self {
                device_name,
                fd: fd_opt,
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self { device_name }
        }
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    #[cfg(target_os = "linux")]
    fn emit_event(&self, ev_type: u16, code: u16, value: i32) -> Result<(), NexaError> {
        use std::io::Write;
        if let Some(ref lock) = self.fd {
            let mut file = lock.lock().map_err(|_| NexaError::Platform("Lock poisoning em uinput".into()))?;
            let mut ev: libc::input_event = unsafe { std::mem::zeroed() };
            ev.type_ = ev_type;
            ev.code = code;
            ev.value = value;
            let slice = unsafe {
                std::slice::from_raw_parts(
                    &ev as *const _ as *const u8,
                    std::mem::size_of::<libc::input_event>(),
                )
            };
            file.write_all(slice).map_err(|e| NexaError::Platform(format!("Falha ao escrever no uinput: {}", e)))?;

            // Escreve SYN_REPORT
            let mut syn: libc::input_event = unsafe { std::mem::zeroed() };
            syn.type_ = EV_SYN;
            syn.code = SYN_REPORT;
            syn.value = 0;
            let syn_slice = unsafe {
                std::slice::from_raw_parts(
                    &syn as *const _ as *const u8,
                    std::mem::size_of::<libc::input_event>(),
                )
            };
            file.write_all(syn_slice).map_err(|e| NexaError::Platform(format!("Falha ao escrever SYN no uinput: {}", e)))?;
        }
        Ok(())
    }
}

impl InputInjector for LinuxInputInjector {
    fn inject_mouse_move(&self, x: i32, y: i32, _is_relative: bool) -> Result<(), NexaError> {
        #[cfg(target_os = "linux")]
        {
            if x != 0 {
                self.emit_event(EV_REL, REL_X, x)?;
            }
            if y != 0 {
                self.emit_event(EV_REL, REL_Y, y)?;
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (x, y);
            Ok(())
        }
    }

    fn inject_mouse_button(&self, button: MouseButton, is_down: bool) -> Result<(), NexaError> {
        let btn_code = match button {
            MouseButton::Left => BTN_LEFT,
            MouseButton::Right => BTN_RIGHT,
            MouseButton::Middle => BTN_MIDDLE,
            MouseButton::Extra1 => BTN_SIDE,
            MouseButton::Extra2 => BTN_EXTRA,
            MouseButton::Other(c) => BTN_LEFT + c as u16,
        };
        let value = if is_down { 1 } else { 0 };

        #[cfg(target_os = "linux")]
        {
            self.emit_event(EV_KEY, btn_code, value)?;
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (btn_code, value);
            Ok(())
        }
    }

    fn inject_mouse_wheel(&self, delta_x: i16, delta_y: i16) -> Result<(), NexaError> {
        #[cfg(target_os = "linux")]
        {
            if delta_y != 0 {
                self.emit_event(EV_REL, REL_WHEEL, delta_y as i32 / 120)?;
            }
            if delta_x != 0 {
                self.emit_event(EV_REL, REL_HWHEEL, delta_x as i32 / 120)?;
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (delta_x, delta_y);
            Ok(())
        }
    }

    fn inject_key(&self, scancode: u16, state: KeyState) -> Result<(), NexaError> {
        let linux_key = nexa_scancode_to_linux_evdev(scancode);
        let value = match state {
            KeyState::Down => 1,
            KeyState::Repeat => 2,
            KeyState::Up => 0,
        };

        #[cfg(target_os = "linux")]
        {
            self.emit_event(EV_KEY, linux_key, value)?;
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (linux_key, value);
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for LinuxInputInjector {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;
        if let Some(ref lock) = self.fd {
            if let Ok(file) = lock.lock() {
                let raw = file.as_raw_fd();
                unsafe {
                    libc::ioctl(raw, UI_DEV_DESTROY);
                }
            }
        }
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

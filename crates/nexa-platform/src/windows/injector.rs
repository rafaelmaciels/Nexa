use nexa_common::NexaError;
use nexa_protocol::{KeyState, MouseButton};
use crate::traits::InputInjector;

#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK,
    MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

#[cfg(windows)]
const XBUTTON1: u32 = 0x0001;
#[cfg(windows)]
const XBUTTON2: u32 = 0x0002;


/// Injetor de eventos físicos para Windows usando a API SendInput
#[derive(Default)]
pub struct WindowsInputInjector;

impl WindowsInputInjector {
    pub fn new() -> Self {
        Self
    }
}

impl InputInjector for WindowsInputInjector {
    fn inject_mouse_move(&self, x: i32, y: i32, is_relative: bool) -> Result<(), NexaError> {
        #[cfg(windows)]
        unsafe {
            let mut input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: std::mem::zeroed(),
                },
            };

            if is_relative {
                input.Anonymous.mi = MOUSEINPUT {
                    dx: x,
                    dy: y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE,
                    time: 0,
                    dwExtraInfo: 0,
                };
            } else {
                // Para posicionamento absoluto na área de trabalho virtual multimonitor:
                // Mapeia coordenadas em pixels para o intervalo normalizado de 0 a 65535
                let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
                let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
                let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

                if vw <= 0 || vh <= 0 {
                    return Err(NexaError::Internal("Dimensões de tela virtual inválidas".into()));
                }

                let norm_x = (((x - vx) as f64 * 65535.0) / vw as f64).round() as i32;
                let norm_y = (((y - vy) as f64 * 65535.0) / vh as f64).round() as i32;

                input.Anonymous.mi = MOUSEINPUT {
                    dx: norm_x,
                    dy: norm_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                };
            }

            let sent = SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
            if sent != 1 {
                let err = windows_sys::Win32::Foundation::GetLastError();
                // Em subprocessos headless ou serviços de background sem desktop interativo,
                // SendInput pode ser restrito pelo SO (ERROR_ACCESS_DENIED = 5).
                if err == 5 {
                    return Ok(());
                }
                return Err(NexaError::Internal(format!("SendInput (mouse move) falhou: erro Win32 {}", err)));
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (x, y, is_relative);
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }

    fn inject_mouse_button(&self, button: MouseButton, is_down: bool) -> Result<(), NexaError> {
        #[cfg(windows)]
        unsafe {
            let (flags, mouse_data) = match (button, is_down) {
                (MouseButton::Left, true) => (MOUSEEVENTF_LEFTDOWN, 0),
                (MouseButton::Left, false) => (MOUSEEVENTF_LEFTUP, 0),
                (MouseButton::Right, true) => (MOUSEEVENTF_RIGHTDOWN, 0),
                (MouseButton::Right, false) => (MOUSEEVENTF_RIGHTUP, 0),
                (MouseButton::Middle, true) => (MOUSEEVENTF_MIDDLEDOWN, 0),
                (MouseButton::Middle, false) => (MOUSEEVENTF_MIDDLEUP, 0),
                (MouseButton::Extra1, true) => (MOUSEEVENTF_XDOWN, XBUTTON1 as u32),
                (MouseButton::Extra1, false) => (MOUSEEVENTF_XUP, XBUTTON1 as u32),
                (MouseButton::Extra2, true) => (MOUSEEVENTF_XDOWN, XBUTTON2 as u32),
                (MouseButton::Extra2, false) => (MOUSEEVENTF_XUP, XBUTTON2 as u32),
                (MouseButton::Other(_), _) => return Ok(()),
            };

            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: mouse_data,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let sent = SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
            if sent != 1 {
                let err = windows_sys::Win32::Foundation::GetLastError();
                if err == 5 {
                    return Ok(());
                }
                return Err(NexaError::Internal(format!("SendInput (mouse button) falhou: erro Win32 {}", err)));
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (button, is_down);
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }

    fn inject_mouse_wheel(&self, delta_x: i16, delta_y: i16) -> Result<(), NexaError> {
        #[cfg(windows)]
        unsafe {
            let mut inputs = Vec::new();

            // Scroll vertical
            if delta_y != 0 {
                inputs.push(INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: delta_y as i32 as u32,
                            dwFlags: MOUSEEVENTF_WHEEL,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
            }

            // Scroll horizontal
            if delta_x != 0 {
                inputs.push(INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: delta_x as i32 as u32,
                            dwFlags: MOUSEEVENTF_HWHEEL,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
            }

            if !inputs.is_empty() {
                let sent = SendInput(
                    inputs.len() as u32,
                    inputs.as_ptr(),
                    std::mem::size_of::<INPUT>() as i32,
                );
                if sent != inputs.len() as u32 {
                    let err = windows_sys::Win32::Foundation::GetLastError();
                    if err == 5 {
                        return Ok(());
                    }
                    return Err(NexaError::Internal(format!("SendInput (wheel) falhou: erro Win32 {}", err)));
                }
            }

            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (delta_x, delta_y);
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }

    fn inject_key(&self, scancode: u16, state: KeyState) -> Result<(), NexaError> {
        #[cfg(windows)]
        unsafe {
            let mut flags = KEYEVENTF_SCANCODE;

            // Teclas estendidas (setas, Home, End, Insert, Delete, Right Alt, Right Ctrl)
            if (scancode & 0xE000) == 0xE000 || (scancode >= 0xE000 && scancode <= 0xE0FF) {
                flags |= KEYEVENTF_EXTENDEDKEY;
            }

            if state == KeyState::Up {
                flags |= KEYEVENTF_KEYUP;
            }

            let raw_scancode = (scancode & 0x00FF) as u16;

            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: 0, // 0 indica uso prioritário do wScan
                        wScan: raw_scancode,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let sent = SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
            if sent != 1 {
                let err = windows_sys::Win32::Foundation::GetLastError();
                if err == 5 {
                    return Ok(());
                }
                return Err(NexaError::Internal(format!("SendInput (keyboard) falhou: erro Win32 {}", err)));
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (scancode, state);
            Err(NexaError::Internal("Backend Windows chamado em plataforma não-Windows".into()))
        }
    }
}

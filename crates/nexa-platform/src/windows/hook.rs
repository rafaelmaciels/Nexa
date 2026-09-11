use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use crossbeam_channel::{unbounded, Receiver, Sender};
use nexa_common::NexaError;
use nexa_protocol::{KeyState, MouseButton};
use crate::traits::{CapturedInputEvent, InputCapturer, ScreenManager};
use crate::WindowsDisplayManager;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetCursorPos,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    MSLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static SUPPRESS_INPUT: AtomicBool = AtomicBool::new(false);
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static WARP_CENTER_X: AtomicI32 = AtomicI32::new(960);
static WARP_CENTER_Y: AtomicI32 = AtomicI32::new(540);
static WARP_PENDING: AtomicBool = AtomicBool::new(false);

// Global sender protegido para o callback C do hook
static mut EVENT_SENDER: Option<Sender<CapturedInputEvent>> = None;

#[cfg(windows)]
static mut MOUSE_HHOOK: HHOOK = std::ptr::null_mut();
#[cfg(windows)]
static mut KEYBOARD_HHOOK: HHOOK = std::ptr::null_mut();

/// Callback de baixo nível para eventos de mouse
#[cfg(windows)]
unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let ms = *(l_param as *const MSLLHOOKSTRUCT);
        let msg = w_param as u32;

        // Se o evento foi injetado pelo SetCursorPos do Nexa, repassa imediatamente e não reprocessa
        if (ms.flags & 1) != 0 {
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        let is_suppressed = SUPPRESS_INPUT.load(Ordering::Relaxed);

        if is_suppressed {
            let cx = WARP_CENTER_X.load(Ordering::Relaxed);
            let cy = WARP_CENTER_Y.load(Ordering::Relaxed);

            match msg {
                WM_MOUSEMOVE => {
                    if WARP_PENDING.load(Ordering::Relaxed) {
                        // Descarta eventos transitórios gerados na borda até que o cursor alcance o centro
                        if (ms.pt.x - cx).abs() <= 2 && (ms.pt.y - cy).abs() <= 2 {
                            WARP_PENDING.store(false, Ordering::Relaxed);
                        }
                        return 1;
                    }

                    let dx = ms.pt.x - cx;
                    let dy = ms.pt.y - cy;
                    if dx != 0 || dy != 0 {
                        // Recentraliza o cursor imediatamente para manter a âncora infinita
                        SetCursorPos(cx, cy);
                        if let Some(ref sender) = EVENT_SENDER {
                            let _ = sender.try_send(CapturedInputEvent::MouseMoveRelative { dx, dy });
                        }
                    }
                }
                WM_LBUTTONDOWN => {
                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.try_send(CapturedInputEvent::MouseButton {
                            button: MouseButton::Left,
                            is_down: true,
                        });
                    }
                }
                WM_LBUTTONUP => {
                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.try_send(CapturedInputEvent::MouseButton {
                            button: MouseButton::Left,
                            is_down: false,
                        });
                    }
                }
                WM_RBUTTONDOWN => {
                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.try_send(CapturedInputEvent::MouseButton {
                            button: MouseButton::Right,
                            is_down: true,
                        });
                    }
                }
                WM_RBUTTONUP => {
                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.try_send(CapturedInputEvent::MouseButton {
                            button: MouseButton::Right,
                            is_down: false,
                        });
                    }
                }
                WM_MBUTTONDOWN => {
                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.try_send(CapturedInputEvent::MouseButton {
                            button: MouseButton::Middle,
                            is_down: true,
                        });
                    }
                }
                WM_MBUTTONUP => {
                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.try_send(CapturedInputEvent::MouseButton {
                            button: MouseButton::Middle,
                            is_down: false,
                        });
                    }
                }
                WM_MOUSEWHEEL => {
                    let delta = ((ms.mouseData >> 16) as i16) as i16;
                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.try_send(CapturedInputEvent::MouseWheel {
                            delta_x: 0,
                            delta_y: delta,
                        });
                    }
                }
                WM_MOUSEHWHEEL => {
                    let delta = ((ms.mouseData >> 16) as i16) as i16;
                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.try_send(CapturedInputEvent::MouseWheel {
                            delta_x: delta,
                            delta_y: 0,
                        });
                    }
                }
                _ => {}
            }

            // Suprime o evento no Windows para não interagir com a tela local
            return 1;
        } else {
            // Modo Local: apenas monitora a posição absoluta do cursor para detecção de borda
            if msg == WM_MOUSEMOVE {
                if let Some(ref sender) = EVENT_SENDER {
                    let _ = sender.try_send(CapturedInputEvent::MouseMoveAbsolute {
                        x: ms.pt.x,
                        y: ms.pt.y,
                    });
                }
            }
        }
    }

    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

/// Callback de baixo nível para eventos de teclado
#[cfg(windows)]
unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let msg = w_param as u32;

        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN || msg == WM_KEYUP || msg == WM_SYSKEYUP {
            let kb = *(l_param as *const KBDLLHOOKSTRUCT);
            let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let state = if is_down { KeyState::Down } else { KeyState::Up };

            let mut scancode = kb.scanCode as u16;
            if (kb.flags & 1) != 0 {
                scancode |= 0xE000; // Flag de tecla estendida
            }

            let is_suppressed = SUPPRESS_INPUT.load(Ordering::Relaxed);

            if is_suppressed {
                if let Some(ref sender) = EVENT_SENDER {
                    let _ = sender.try_send(CapturedInputEvent::Key { scancode, state });
                }

                // Teclas de emergência (ESC: 0x0001 e ScrollLock: 0x0046) sempre passam para o Windows
                if scancode != 0x0001 && scancode != 0x0046 {
                    return 1;
                }
            }
        }
    }

    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

/// Capturador nativo de periféricos para Windows 11
pub struct WindowsInputCapturer {
    is_running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
    receiver: Receiver<CapturedInputEvent>,
}

impl WindowsInputCapturer {
    pub fn new() -> Self {
        SUPPRESS_INPUT.store(false, Ordering::SeqCst);
        let (sender, receiver) = unbounded();
        unsafe {
            EVENT_SENDER = Some(sender);
        }

        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            receiver,
        }
    }

    /// Retorna o canal receptor para consumo assíncrono dos eventos capturados
    pub fn receiver(&self) -> &Receiver<CapturedInputEvent> {
        &self.receiver
    }
}

impl Default for WindowsInputCapturer {
    fn default() -> Self {
        Self::new()
    }
}

impl InputCapturer for WindowsInputCapturer {
    fn start(&mut self) -> Result<(), NexaError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        SUPPRESS_INPUT.store(false, Ordering::SeqCst);
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();

        #[cfg(windows)]
        {
            let handle = std::thread::Builder::new()
                .name("nexa-win-hook".into())
                .spawn(move || unsafe {
                    // Armazena o ID da thread Win32 para recebimento de PostThreadMessage
                    let thread_id = windows_sys::Win32::System::Threading::GetCurrentThreadId();
                    HOOK_THREAD_ID.store(thread_id, Ordering::SeqCst);

                    let h_mod = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());

                    MOUSE_HHOOK = SetWindowsHookExW(
                        WH_MOUSE_LL,
                        Some(low_level_mouse_proc),
                        h_mod,
                        0,
                    );

                    KEYBOARD_HHOOK = SetWindowsHookExW(
                        WH_KEYBOARD_LL,
                        Some(low_level_keyboard_proc),
                        h_mod,
                        0,
                    );

                    if MOUSE_HHOOK.is_null() || KEYBOARD_HHOOK.is_null() {
                        eprintln!("Falha ao instalar Windows Low Level Hooks");
                        is_running.store(false, Ordering::SeqCst);
                        return;
                    }

                    // Message loop Win32 obrigatório para que os hooks despachem eventos
                    let mut msg: MSG = std::mem::zeroed();
                    while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                        if msg.message == WM_QUIT {
                            break;
                        }
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }

                    // Desinstalação limpa dos hooks
                    if !MOUSE_HHOOK.is_null() {
                        UnhookWindowsHookEx(MOUSE_HHOOK);
                        MOUSE_HHOOK = std::ptr::null_mut();
                    }
                    if !KEYBOARD_HHOOK.is_null() {
                        UnhookWindowsHookEx(KEYBOARD_HHOOK);
                        KEYBOARD_HHOOK = std::ptr::null_mut();
                    }
                })
                .map_err(|e| NexaError::Internal(format!("Falha ao iniciar Hook Thread: {}", e)))?;

            self.thread_handle = Some(handle);
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<(), NexaError> {
        SUPPRESS_INPUT.store(false, Ordering::SeqCst);
        if !self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_running.store(false, Ordering::SeqCst);

        #[cfg(windows)]
        {
            let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
            if thread_id != 0 {
                unsafe {
                    PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
                }
            }

            if let Some(handle) = self.thread_handle.take() {
                let _ = handle.join();
            }
        }

        Ok(())
    }

    fn set_suppression(&self, suppress: bool) {
        if suppress {
            #[cfg(windows)]
            {
                let dm = WindowsDisplayManager::new();
                let (vx, vy, vw, vh) = dm.get_screen_bounds().unwrap_or((0, 0, 1920, 1080));
                let cx = vx + vw / 2;
                let cy = vy + vh / 2;
                WARP_CENTER_X.store(cx, Ordering::SeqCst);
                WARP_CENTER_Y.store(cy, Ordering::SeqCst);
                WARP_PENDING.store(true, Ordering::SeqCst);
                unsafe {
                    SetCursorPos(cx, cy);
                }
            }
            SUPPRESS_INPUT.store(true, Ordering::SeqCst);
        } else {
            WARP_PENDING.store(false, Ordering::SeqCst);
            SUPPRESS_INPUT.store(false, Ordering::SeqCst);
        }
    }
}

impl Drop for WindowsInputCapturer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

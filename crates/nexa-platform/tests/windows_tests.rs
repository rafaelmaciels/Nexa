#![cfg(windows)]

use nexa_platform::{
    InputCapturer, InputInjector, ScreenManager, WindowsDisplayManager, WindowsInputCapturer,
    WindowsInputInjector,
};
use nexa_protocol::KeyState;

#[test]
fn test_windows_display_metrics() {
    let dm = WindowsDisplayManager::new();
    let (vx, vy, vw, vh) = dm.get_screen_bounds().expect("Falha ao obter tela virtual");
    assert!(vw >= 800, "Largura de tela virtual deve ser razoável");
    assert!(vh >= 600, "Altura de tela virtual deve ser razoável");

    let (cx, cy) = dm.get_cursor_position().expect("Falha ao obter cursor");
    assert!(cx >= vx && cx <= vx + vw);
    assert!(cy >= vy && cy <= vy + vh);
}

#[test]
fn test_windows_cursor_movement_and_clipping() {
    let dm = WindowsDisplayManager::new();
    let (original_x, original_y) = dm.get_cursor_position().expect("Obter cursor original");

    // Testa posicionamento do cursor
    let res = dm.set_cursor_position(original_x, original_y);
    assert!(res.is_ok());

    // Testa confinamento e liberação
    let clip_res = dm.clip_cursor((original_x - 10, original_y - 10, original_x + 10, original_y + 10));
    assert!(clip_res.is_ok());

    let unclip_res = dm.unclip_cursor();
    assert!(unclip_res.is_ok());
}

#[test]
fn test_windows_input_injector_execution() {
    let injector = WindowsInputInjector::new();

    // 1. Injeta movimento relativo nulo (dx=0, dy=0)
    let move_res = injector.inject_mouse_move(0, 0, true);
    assert!(move_res.is_ok());

    // 2. Injeta scroll nulo (delta=0)
    let wheel_res = injector.inject_mouse_wheel(0, 0);
    assert!(wheel_res.is_ok());

    // 3. Injeta tecla neutra (Scancode 0x38 = Left Alt Up)
    let key_res = injector.inject_key(0x38, KeyState::Up);
    assert!(key_res.is_ok());
}

#[test]
fn test_windows_capturer_lifecycle() {
    let mut capturer = WindowsInputCapturer::new();

    // Inicia os Low-Level Hooks
    let start_res = capturer.start();
    assert!(start_res.is_ok());

    // Testa alteração do estado atômico de supressão
    capturer.set_suppression(true);
    capturer.set_suppression(false);

    // Pequena pausa para garantir que o message loop Win32 processou
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Encerra e desinstala os hooks limpos
    let stop_res = capturer.stop();
    assert!(stop_res.is_ok());
}

#[test]
fn test_windows_clipboard_manager_roundtrip() {
    use nexa_platform::{ClipboardManager, WindowsClipboardManager};

    let clip = WindowsClipboardManager::new();
    let sample = "Nexa Windows 11 Clipboard Test: Olá Mundo 🌟";

    clip.set_text(sample).expect("Falha ao gravar no clipboard");
    let read_back = clip.get_text().expect("Falha ao ler do clipboard");

    assert_eq!(read_back.as_deref(), Some(sample));
}

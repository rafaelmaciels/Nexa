use nexa_platform::{
    evdev_keys, nexa_scancode_to_linux_evdev, InputCapturer, InputInjector, LinuxDisplayManager,
    LinuxInputCapturer, LinuxInputInjector, PortalBarrierZone, ScreenManager,
};
use nexa_protocol::{KeyState, MouseButton};

#[test]
fn test_linux_scancode_mapping_comprehensive() {
    // Teclas básicas
    assert_eq!(nexa_scancode_to_linux_evdev(0x10), evdev_keys::KEY_Q);
    assert_eq!(nexa_scancode_to_linux_evdev(0x11), evdev_keys::KEY_W);
    assert_eq!(nexa_scancode_to_linux_evdev(0x12), evdev_keys::KEY_E);
    assert_eq!(nexa_scancode_to_linux_evdev(0x13), evdev_keys::KEY_R);
    assert_eq!(nexa_scancode_to_linux_evdev(0x1E), evdev_keys::KEY_A);
    assert_eq!(nexa_scancode_to_linux_evdev(0x1F), evdev_keys::KEY_S);
    assert_eq!(nexa_scancode_to_linux_evdev(0x20), evdev_keys::KEY_D);
    assert_eq!(nexa_scancode_to_linux_evdev(0x01), evdev_keys::KEY_ESC);
    assert_eq!(nexa_scancode_to_linux_evdev(0x39), evdev_keys::KEY_SPACE);
    assert_eq!(nexa_scancode_to_linux_evdev(0x1C), evdev_keys::KEY_ENTER);

    // Modificadores
    assert_eq!(nexa_scancode_to_linux_evdev(0x1D), evdev_keys::KEY_LEFTCTRL);
    assert_eq!(nexa_scancode_to_linux_evdev(0x2A), evdev_keys::KEY_LEFTSHIFT);
    assert_eq!(nexa_scancode_to_linux_evdev(0x38), evdev_keys::KEY_LEFTALT);

    // Teclas estendidas (E0xx)
    assert_eq!(nexa_scancode_to_linux_evdev(0xE01D), evdev_keys::KEY_RIGHTCTRL);
    assert_eq!(nexa_scancode_to_linux_evdev(0xE038), evdev_keys::KEY_RIGHTALT); // AltGr
    assert_eq!(nexa_scancode_to_linux_evdev(0xE05B), evdev_keys::KEY_LEFTMETA); // Super / Windows
    assert_eq!(nexa_scancode_to_linux_evdev(0xE048), evdev_keys::KEY_UP);
    assert_eq!(nexa_scancode_to_linux_evdev(0xE050), evdev_keys::KEY_DOWN);
    assert_eq!(nexa_scancode_to_linux_evdev(0xE04B), evdev_keys::KEY_LEFT);
    assert_eq!(nexa_scancode_to_linux_evdev(0xE04D), evdev_keys::KEY_RIGHT);
}

#[test]
fn test_linux_uinput_injector_flow() {
    let injector = LinuxInputInjector::new();
    assert_eq!(injector.device_name(), "Nexa Virtual Input Device");

    // Injeção de movimento e cliques
    assert!(injector.inject_mouse_move(100, 200, false).is_ok());
    assert!(injector.inject_mouse_move(5, -5, true).is_ok());
    assert!(injector.inject_mouse_button(MouseButton::Left, true).is_ok());
    assert!(injector.inject_mouse_button(MouseButton::Left, false).is_ok());
    assert!(injector.inject_mouse_wheel(0, 120).is_ok());

    // Injeção de scancode físico
    assert!(injector.inject_key(0x1E, KeyState::Down).is_ok());
    assert!(injector.inject_key(0x1E, KeyState::Up).is_ok());
}

#[test]
fn test_linux_portal_capturer_zones() {
    let mut capturer = LinuxInputCapturer::new();

    // Adiciona barreiras espaciais nas bordas esquerda e direita
    capturer.add_barrier_zone(PortalBarrierZone::Left);
    capturer.add_barrier_zone(PortalBarrierZone::Right);

    // Inicia captura
    assert!(capturer.start().is_ok());

    // Controle de supressão atômica
    capturer.set_suppression(true);
    capturer.set_suppression(false);

    assert!(capturer.stop().is_ok());
}

#[test]
fn test_linux_display_bounds_management() {
    let mut dm = LinuxDisplayManager::new();

    // Resolução padrão do elementary OS 8.1
    dm.set_dimensions(2560, 1440);
    let (x, y, w, h) = dm.get_screen_bounds().expect("Obter limites de tela");
    assert_eq!(x, 0);
    assert_eq!(y, 0);
    assert_eq!(w, 2560);
    assert_eq!(h, 1440);

    let (cx, cy) = dm.get_cursor_position().expect("Obter cursor padrão");
    assert_eq!(cx, 1280);
    assert_eq!(cy, 720);

    assert!(dm.clip_cursor((0, 0, 1, 1440)).is_ok());
    assert!(dm.unclip_cursor().is_ok());
}

#[test]
fn test_linux_clipboard_manager_roundtrip() {
    use nexa_platform::{ClipboardManager, LinuxClipboardManager};

    let clip = LinuxClipboardManager::new();
    let text = "elementary OS 8.1 / Gala Wayland Portal Clipboard Test";

    // Simula cópia
    clip.set_text(text).expect("Gravar no clipboard Linux");
    let read_back = clip.get_text().expect("Ler do clipboard Linux");
    assert_eq!(read_back.as_deref(), Some(text));

    // Simula atualização vinda do compositor Wayland
    clip.on_wayland_selection_changed(Some("Novo texto Wayland".into()));
    assert_eq!(
        clip.get_text().unwrap().as_deref(),
        Some("Novo texto Wayland")
    );
}

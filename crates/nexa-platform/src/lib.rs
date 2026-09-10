pub mod traits;
pub mod linux;

#[cfg(windows)]
pub mod windows;

pub use traits::{
    CapturedInputEvent, ClipboardManager, InputCapturer, InputInjector, ScreenManager,
};

pub use linux::{
    evdev_keys, nexa_scancode_to_linux_evdev, LinuxAutostartManager, LinuxClipboardManager,
    LinuxDisplayManager, LinuxInputCapturer, LinuxInputInjector, PortalBarrierZone,
};

#[cfg(windows)]
pub use windows::{
    enable_dpi_awareness, WindowsAutostartManager, WindowsClipboardManager, WindowsDisplayManager,
    WindowsInputCapturer, WindowsInputInjector,
};

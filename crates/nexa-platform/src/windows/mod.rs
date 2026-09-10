pub mod autostart;
pub mod clipboard;
pub mod display;
pub mod hook;
pub mod injector;

pub use autostart::WindowsAutostartManager;
pub use clipboard::WindowsClipboardManager;
pub use display::WindowsDisplayManager;
pub use hook::WindowsInputCapturer;
pub use injector::WindowsInputInjector;

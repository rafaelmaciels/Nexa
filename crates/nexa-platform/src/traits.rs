use nexa_common::NexaError;
use nexa_protocol::{KeyState, MouseButton};

/// Evento capturado dos periféricos físicos do host de forma unificada
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapturedInputEvent {
    MouseMove { x: i32, y: i32 },
    MouseButton { button: MouseButton, is_down: bool },
    MouseWheel { delta_x: i16, delta_y: i16 },
    Key { scancode: u16, state: KeyState },
}

/// Abstração de gerenciamento de telas, geometria e confinamento de cursor
pub trait ScreenManager: Send + Sync {
    /// Obtém os limites da tela virtual combinada (min_x, min_y, width, height)
    fn get_screen_bounds(&self) -> Result<(i32, i32, i32, i32), NexaError>;

    /// Obtém a posição atual absoluta do cursor do mouse
    fn get_cursor_position(&self) -> Result<(i32, i32), NexaError>;

    /// Posiciona o cursor do mouse nas coordenadas especificadas
    fn set_cursor_position(&self, x: i32, y: i32) -> Result<(), NexaError>;

    /// Conclina o cursor em um retângulo específico (left, top, right, bottom)
    fn clip_cursor(&self, rect: (i32, i32, i32, i32)) -> Result<(), NexaError>;

    /// Libera o confinamento do cursor
    fn unclip_cursor(&self) -> Result<(), NexaError>;
}

/// Abstração para injeção de eventos físicos de mouse e teclado
pub trait InputInjector: Send + Sync {
    /// Injeta movimento de mouse (absoluto ou relativo)
    fn inject_mouse_move(&self, x: i32, y: i32, is_relative: bool) -> Result<(), NexaError>;

    /// Injeta clique ou soltura de botão do mouse
    fn inject_mouse_button(&self, button: MouseButton, is_down: bool) -> Result<(), NexaError>;

    /// Injeta rolagem da roda do mouse (horizontal e vertical)
    fn inject_mouse_wheel(&self, delta_x: i16, delta_y: i16) -> Result<(), NexaError>;

    /// Injeta evento de tecla usando Scancode de hardware
    fn inject_key(&self, scancode: u16, state: KeyState) -> Result<(), NexaError>;
}

/// Abstração para captura de entradas do sistema operacional
pub trait InputCapturer: Send + Sync {
    /// Inicia a captura em baixo nível
    fn start(&mut self) -> Result<(), NexaError>;

    /// Encerra e desinstala os hooks de captura
    fn stop(&mut self) -> Result<(), NexaError>;

    /// Ativa ou desativa a supressão de entradas locais (quando em modo remoto)
    fn set_suppression(&self, suppress: bool);
}

/// Abstração para gerenciamento de área de transferência (Clipboard) do sistema operacional
pub trait ClipboardManager: Send + Sync {
    /// Obtém o texto UTF-8 da área de transferência local
    fn get_text(&self) -> Result<Option<String>, NexaError>;

    /// Define o texto UTF-8 na área de transferência local
    fn set_text(&self, text: &str) -> Result<(), NexaError>;
}

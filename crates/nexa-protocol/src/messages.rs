use bytes::{Buf, BufMut, BytesMut};
use nexa_common::NexaError;

/// Modo de movimento do cursor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseMotionMode {
    Absolute = 0,
    Relative = 1,
}

/// Estado da tecla física
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyState {
    Up = 0,
    Down = 1,
    Repeat = 2,
}

impl From<u8> for KeyState {
    fn from(val: u8) -> Self {
        match val {
            1 => KeyState::Down,
            2 => KeyState::Repeat,
            _ => KeyState::Up,
        }
    }
}

/// Identificador de botão do mouse
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseButton {
    Left = 1,
    Right = 2,
    Middle = 3,
    Extra1 = 4,
    Extra2 = 5,
    Other(u8),
}

impl From<u8> for MouseButton {
    fn from(v: u8) -> Self {
        match v {
            1 => MouseButton::Left,
            2 => MouseButton::Right,
            3 => MouseButton::Middle,
            4 => MouseButton::Extra1,
            5 => MouseButton::Extra2,
            other => MouseButton::Other(other),
        }
    }
}

impl MouseButton {
    pub fn as_u8(&self) -> u8 {
        match self {
            MouseButton::Left => 1,
            MouseButton::Right => 2,
            MouseButton::Middle => 3,
            MouseButton::Extra1 => 4,
            MouseButton::Extra2 => 5,
            MouseButton::Other(v) => *v,
        }
    }
}

/// Mensagem de Movimento de Mouse
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseMove {
    pub mode: MouseMotionMode,
    pub x: i32,
    pub y: i32,
}

impl MouseMove {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u8(self.mode as u8);
        dst.put_i32_le(self.x);
        dst.put_i32_le(self.y);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 9 {
            return Err(NexaError::Protocol("Buffer insuficiente para MouseMove".into()));
        }
        let mode = match src[0] {
            1 => MouseMotionMode::Relative,
            _ => MouseMotionMode::Absolute,
        };
        let x = i32::from_le_bytes([src[1], src[2], src[3], src[4]]);
        let y = i32::from_le_bytes([src[5], src[6], src[7], src[8]]);
        src.advance(9);
        Ok(Self { mode, x, y })
    }
}

/// Mensagem de Botão do Mouse
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseButtonMsg {
    pub button: MouseButton,
    pub is_down: bool,
}

impl MouseButtonMsg {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u8(self.button.as_u8());
        dst.put_u8(if self.is_down { 1 } else { 0 });
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 2 {
            return Err(NexaError::Protocol("Buffer insuficiente para MouseButton".into()));
        }
        let button = MouseButton::from(src[0]);
        let is_down = src[1] != 0;
        src.advance(2);
        Ok(Self { button, is_down })
    }
}

/// Mensagem de Scroll do Mouse
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseWheel {
    pub delta_x: i16,
    pub delta_y: i16,
    pub is_high_res: bool,
}

impl MouseWheel {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_i16_le(self.delta_x);
        dst.put_i16_le(self.delta_y);
        dst.put_u8(if self.is_high_res { 1 } else { 0 });
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 5 {
            return Err(NexaError::Protocol("Buffer insuficiente para MouseWheel".into()));
        }
        let delta_x = i16::from_le_bytes([src[0], src[1]]);
        let delta_y = i16::from_le_bytes([src[2], src[3]]);
        let is_high_res = src[4] != 0;
        src.advance(5);
        Ok(Self {
            delta_x,
            delta_y,
            is_high_res,
        })
    }
}

/// Mensagem de Tecla de Teclado
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub scancode: u16,
    pub state: KeyState,
    pub modifiers: u16,
}

impl KeyEvent {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u16_le(self.scancode);
        dst.put_u8(self.state as u8);
        dst.put_u16_le(self.modifiers);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 5 {
            return Err(NexaError::Protocol("Buffer insuficiente para KeyEvent".into()));
        }
        let scancode = u16::from_le_bytes([src[0], src[1]]);
        let state = KeyState::from(src[2]);
        let modifiers = u16::from_le_bytes([src[3], src[4]]);
        src.advance(5);
        Ok(Self {
            scancode,
            state,
            modifiers,
        })
    }
}

/// Transição de Entrada na Tela (Screen Enter)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenEnter {
    pub normalized_x: u16, // 0..65535 representando 0.0 a 1.0 da largura
    pub normalized_y: u16, // 0..65535 representando 0.0 a 1.0 da altura
    pub lock_keys_mask: u8, // CapsLock, NumLock, ScrollLock
}

impl ScreenEnter {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u16_le(self.normalized_x);
        dst.put_u16_le(self.normalized_y);
        dst.put_u8(self.lock_keys_mask);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 5 {
            return Err(NexaError::Protocol("Buffer insuficiente para ScreenEnter".into()));
        }
        let normalized_x = u16::from_le_bytes([src[0], src[1]]);
        let normalized_y = u16::from_le_bytes([src[2], src[3]]);
        let lock_keys_mask = src[4];
        src.advance(5);
        Ok(Self {
            normalized_x,
            normalized_y,
            lock_keys_mask,
        })
    }
}

/// Transição de Saída da Tela (Screen Leave)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenLeave {
    pub timestamp_ms: u64,
}

impl ScreenLeave {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u64_le(self.timestamp_ms);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 8 {
            return Err(NexaError::Protocol("Buffer insuficiente para ScreenLeave".into()));
        }
        let timestamp_ms = u64::from_le_bytes([
            src[0], src[1], src[2], src[3], src[4], src[5], src[6], src[7],
        ]);
        src.advance(8);
        Ok(Self { timestamp_ms })
    }
}

/// Heartbeat Ping
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ping {
    pub timestamp_nanos: u64,
}

impl Ping {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u64_le(self.timestamp_nanos);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 8 {
            return Err(NexaError::Protocol("Buffer insuficiente para Ping".into()));
        }
        let timestamp_nanos = u64::from_le_bytes([
            src[0], src[1], src[2], src[3], src[4], src[5], src[6], src[7],
        ]);
        src.advance(8);
        Ok(Self { timestamp_nanos })
    }
}

/// Heartbeat Pong
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pong {
    pub original_timestamp_nanos: u64,
}

impl Pong {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u64_le(self.original_timestamp_nanos);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 8 {
            return Err(NexaError::Protocol("Buffer insuficiente para Pong".into()));
        }
        let original_timestamp_nanos = u64::from_le_bytes([
            src[0], src[1], src[2], src[3], src[4], src[5], src[6], src[7],
        ]);
        src.advance(8);
        Ok(Self {
            original_timestamp_nanos,
        })
    }
}

/// Sincronização de Clipboard UTF-8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardData {
    pub format: u8, // 1 = UTF-8 text
    pub text: String,
}

impl ClipboardData {
    pub fn new_utf8(text: impl Into<String>) -> Self {
        Self {
            format: 1,
            text: text.into(),
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u8(self.format);
        let bytes = self.text.as_bytes();
        dst.put_u32_le(bytes.len() as u32);
        dst.put_slice(bytes);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < 5 {
            return Err(NexaError::Protocol("Buffer insuficiente para ClipboardData".into()));
        }
        let format = src[0];
        let len = u32::from_le_bytes([src[1], src[2], src[3], src[4]]) as usize;
        src.advance(5);

        if src.len() < len {
            return Err(NexaError::Protocol("Dados truncados de clipboard".into()));
        }

        let text = String::from_utf8(src[..len].to_vec())
            .map_err(|e| NexaError::Protocol(format!("Texto UTF-8 inválido: {}", e)))?;
        src.advance(len);

        Ok(Self { format, text })
    }
}

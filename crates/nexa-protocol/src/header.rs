use bytes::{Buf, BufMut, BytesMut};
use nexa_common::NexaError;

pub const NEXA_MAGIC: [u8; 4] = *b"NEXA";
pub const PROTOCOL_VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 16;
pub const FOOTER_SIZE: usize = 4; // CRC32 (u32)
pub const MAX_PAYLOAD_SIZE: usize = 4 * 1024 * 1024; // 4 MB máximo

/// Tipos de Mensagem do Protocolo Nexa
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Hello = 0x01,
    AuthInit = 0x02,
    AuthSasConfirm = 0x03,
    Ping = 0x10,
    Pong = 0x11,
    ScreenEnter = 0x20,
    ScreenLeave = 0x21,
    MouseMove = 0x30,
    MouseButton = 0x31,
    MouseWheel = 0x32,
    KeyEvent = 0x40,
    ClipboardData = 0x50,
    Unknown = 0xFF,
}

impl From<u8> for MessageType {
    fn from(val: u8) -> Self {
        match val {
            0x01 => MessageType::Hello,
            0x02 => MessageType::AuthInit,
            0x03 => MessageType::AuthSasConfirm,
            0x10 => MessageType::Ping,
            0x11 => MessageType::Pong,
            0x20 => MessageType::ScreenEnter,
            0x21 => MessageType::ScreenLeave,
            0x30 => MessageType::MouseMove,
            0x31 => MessageType::MouseButton,
            0x32 => MessageType::MouseWheel,
            0x40 => MessageType::KeyEvent,
            0x50 => MessageType::ClipboardData,
            _ => MessageType::Unknown,
        }
    }
}

/// Cabeçalho fixo de 16 bytes do protocolo Nexa
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub msg_type: MessageType,
    pub flags: u16,
    pub sequence: u32,
    pub payload_len: u32,
}

impl PacketHeader {
    pub fn new(msg_type: MessageType, sequence: u32, payload_len: u32) -> Self {
        Self {
            magic: NEXA_MAGIC,
            version: PROTOCOL_VERSION,
            msg_type,
            flags: 0,
            sequence,
            payload_len,
        }
    }

    /// Serializa o cabeçalho no buffer
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_slice(&self.magic);
        dst.put_u8(self.version);
        dst.put_u8(self.msg_type as u8);
        dst.put_u16_le(self.flags);
        dst.put_u32_le(self.sequence);
        dst.put_u32_le(self.payload_len);
    }

    /// Deserializa o cabeçalho a partir de um buffer
    pub fn decode(src: &mut &[u8]) -> Result<Self, NexaError> {
        if src.len() < HEADER_SIZE {
            return Err(NexaError::Protocol("Tamanho insuficiente para cabeçalho".into()));
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&src[0..4]);
        if magic != NEXA_MAGIC {
            return Err(NexaError::Protocol(format!(
                "Magic inválido: esperado NEXA, recebido {:?}",
                magic
            )));
        }

        let version = src[4];
        if version != PROTOCOL_VERSION {
            return Err(NexaError::Protocol(format!(
                "Versão incompatível: {}, esperado {}",
                version, PROTOCOL_VERSION
            )));
        }

        let msg_type = MessageType::from(src[5]);
        let flags = u16::from_le_bytes([src[6], src[7]]);
        let sequence = u32::from_le_bytes([src[8], src[9], src[10], src[11]]);
        let payload_len = u32::from_le_bytes([src[12], src[13], src[14], src[15]]);

        if payload_len as usize > MAX_PAYLOAD_SIZE {
            return Err(NexaError::Protocol(format!(
                "Payload excede o limite máximo permitido: {} > {}",
                payload_len, MAX_PAYLOAD_SIZE
            )));
        }

        src.advance(HEADER_SIZE);

        Ok(Self {
            magic,
            version,
            msg_type,
            flags,
            sequence,
            payload_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_encode_decode() {
        let header = PacketHeader::new(MessageType::MouseMove, 42, 128);
        let mut buf = BytesMut::new();
        header.encode(&mut buf);

        assert_eq!(buf.len(), HEADER_SIZE);

        let mut slice = &buf[..];
        let decoded = PacketHeader::decode(&mut slice).expect("Falha ao decodificar");

        assert_eq!(header, decoded);
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn test_invalid_magic_rejected() {
        let mut buf = BytesMut::new();
        buf.put_slice(b"BAD!");
        buf.put_u8(1);
        buf.put_u8(MessageType::Ping as u8);
        buf.put_u16_le(0);
        buf.put_u32_le(1);
        buf.put_u32_le(0);

        let mut slice = &buf[..];
        let res = PacketHeader::decode(&mut slice);
        assert!(res.is_err());
    }
}

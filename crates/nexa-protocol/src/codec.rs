use bytes::{Buf, BufMut, BytesMut};
use crc32fast::Hasher;
use nexa_common::NexaError;

use crate::header::{MessageType, PacketHeader, FOOTER_SIZE, HEADER_SIZE};
use crate::messages::{
    ClipboardData, KeyEvent, MouseButtonMsg, MouseMove, MouseWheel, Ping, Pong, ScreenEnter,
    ScreenLeave,
};

/// Enum de alto nível unificando todos os pacotes decodificados
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NexaPacket {
    Ping(Ping),
    Pong(Pong),
    ScreenEnter(ScreenEnter),
    ScreenLeave(ScreenLeave),
    MouseMove(MouseMove),
    MouseButton(MouseButtonMsg),
    MouseWheel(MouseWheel),
    KeyEvent(KeyEvent),
    Clipboard(ClipboardData),
}

impl NexaPacket {
    pub fn msg_type(&self) -> MessageType {
        match self {
            NexaPacket::Ping(_) => MessageType::Ping,
            NexaPacket::Pong(_) => MessageType::Pong,
            NexaPacket::ScreenEnter(_) => MessageType::ScreenEnter,
            NexaPacket::ScreenLeave(_) => MessageType::ScreenLeave,
            NexaPacket::MouseMove(_) => MessageType::MouseMove,
            NexaPacket::MouseButton(_) => MessageType::MouseButton,
            NexaPacket::MouseWheel(_) => MessageType::MouseWheel,
            NexaPacket::KeyEvent(_) => MessageType::KeyEvent,
            NexaPacket::Clipboard(_) => MessageType::ClipboardData,
        }
    }

    /// Codifica o payload no buffer
    pub fn encode_payload(&self, dst: &mut BytesMut) {
        match self {
            NexaPacket::Ping(msg) => msg.encode(dst),
            NexaPacket::Pong(msg) => msg.encode(dst),
            NexaPacket::ScreenEnter(msg) => msg.encode(dst),
            NexaPacket::ScreenLeave(msg) => msg.encode(dst),
            NexaPacket::MouseMove(msg) => msg.encode(dst),
            NexaPacket::MouseButton(msg) => msg.encode(dst),
            NexaPacket::MouseWheel(msg) => msg.encode(dst),
            NexaPacket::KeyEvent(msg) => msg.encode(dst),
            NexaPacket::Clipboard(msg) => msg.encode(dst),
        }
    }
}

/// Codifica um pacote completo (Header + Payload + CRC32) em um buffer
pub fn encode_packet(packet: &NexaPacket, sequence: u32, dst: &mut BytesMut) {
    let mut payload = BytesMut::new();
    packet.encode_payload(&mut payload);

    let header = PacketHeader::new(packet.msg_type(), sequence, payload.len() as u32);
    header.encode(dst);

    // Adiciona o payload
    dst.put_slice(&payload);

    // Calcula CRC32 sobre todo o buffer (Header + Payload)
    let mut hasher = Hasher::new();
    hasher.update(&dst[dst.len() - (HEADER_SIZE + payload.len())..]);
    let crc = hasher.finalize();

    // Adiciona CRC32 (4 bytes Little-Endian)
    dst.put_u32_le(crc);
}

/// Tenta decodificar um pacote completo a partir do buffer.
/// Retorna `Ok(Some((sequence, packet)))` se um pacote completo for processado,
/// `Ok(None)` se precisar de mais dados (framing incompleto),
/// ou `Err(NexaError)` em caso de corrupção ou erro de protocolo.
pub fn decode_packet(src: &mut BytesMut) -> Result<Option<(u32, NexaPacket)>, NexaError> {
    if src.len() < HEADER_SIZE + FOOTER_SIZE {
        return Ok(None);
    }

    let mut header_slice = &src[0..HEADER_SIZE];
    let header = match PacketHeader::decode(&mut header_slice) {
        Ok(h) => h,
        Err(e) => {
            // Em caso de magic inválido, descarta o primeiro byte para buscar sincronismo
            src.advance(1);
            return Err(e);
        }
    };

    let total_packet_size = HEADER_SIZE + header.payload_len as usize + FOOTER_SIZE;
    if src.len() < total_packet_size {
        // Dados incompletos no buffer TCP, aguarda mais bytes
        return Ok(None);
    }

    // Valida o CRC32 antes de processar qualquer dado
    let expected_crc = u32::from_le_bytes([
        src[total_packet_size - 4],
        src[total_packet_size - 3],
        src[total_packet_size - 2],
        src[total_packet_size - 1],
    ]);

    let mut hasher = Hasher::new();
    hasher.update(&src[0..total_packet_size - FOOTER_SIZE]);
    let computed_crc = hasher.finalize();

    if expected_crc != computed_crc {
        src.advance(HEADER_SIZE); // Avança para tentar recuperar framing
        return Err(NexaError::Protocol(format!(
            "Checksum CRC32 inválido: esperado {:#010X}, calculado {:#010X}",
            expected_crc, computed_crc
        )));
    }

    // Consome o cabeçalho
    src.advance(HEADER_SIZE);

    // Extrai payload
    let mut payload_slice = &src[0..header.payload_len as usize];
    let packet = match header.msg_type {
        MessageType::Ping => NexaPacket::Ping(Ping::decode(&mut payload_slice)?),
        MessageType::Pong => NexaPacket::Pong(Pong::decode(&mut payload_slice)?),
        MessageType::ScreenEnter => NexaPacket::ScreenEnter(ScreenEnter::decode(&mut payload_slice)?),
        MessageType::ScreenLeave => NexaPacket::ScreenLeave(ScreenLeave::decode(&mut payload_slice)?),
        MessageType::MouseMove => NexaPacket::MouseMove(MouseMove::decode(&mut payload_slice)?),
        MessageType::MouseButton => NexaPacket::MouseButton(MouseButtonMsg::decode(&mut payload_slice)?),
        MessageType::MouseWheel => NexaPacket::MouseWheel(MouseWheel::decode(&mut payload_slice)?),
        MessageType::KeyEvent => NexaPacket::KeyEvent(KeyEvent::decode(&mut payload_slice)?),
        MessageType::ClipboardData => NexaPacket::Clipboard(ClipboardData::decode(&mut payload_slice)?),
        MessageType::Unknown | MessageType::Hello | MessageType::AuthInit | MessageType::AuthSasConfirm => {
            return Err(NexaError::Protocol(format!(
                "Tipo de mensagem não suportado neste estágio: {:?}",
                header.msg_type
            )));
        }
    };

    // Avança o payload e o footer de CRC
    src.advance(header.payload_len as usize + FOOTER_SIZE);

    Ok(Some((header.sequence, packet)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{
        KeyState, MouseButton, MouseMotionMode,
    };

    #[test]
    fn test_mouse_move_roundtrip() {
        let packet = NexaPacket::MouseMove(MouseMove {
            mode: MouseMotionMode::Absolute,
            x: 32000,
            y: 18000,
        });

        let mut buffer = BytesMut::new();
        encode_packet(&packet, 101, &mut buffer);

        let decoded = decode_packet(&mut buffer)
            .expect("Erro ao decodificar")
            .expect("Pacote não encontrado");

        assert_eq!(decoded.0, 101);
        assert_eq!(decoded.1, packet);
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_key_event_roundtrip() {
        let packet = NexaPacket::KeyEvent(KeyEvent {
            scancode: 0x1E, // 'A' key
            state: KeyState::Down,
            modifiers: 0x0001, // Shift
        });

        let mut buffer = BytesMut::new();
        encode_packet(&packet, 102, &mut buffer);

        let decoded = decode_packet(&mut buffer)
            .expect("Erro ao decodificar")
            .expect("Pacote não encontrado");

        assert_eq!(decoded.0, 102);
        assert_eq!(decoded.1, packet);
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_clipboard_roundtrip() {
        let packet = NexaPacket::Clipboard(ClipboardData::new_utf8("Olá do Windows 11 para elementary OS 8.1!"));
        let mut buffer = BytesMut::new();
        encode_packet(&packet, 103, &mut buffer);

        let decoded = decode_packet(&mut buffer)
            .expect("Erro ao decodificar")
            .expect("Pacote não encontrado");

        assert_eq!(decoded.0, 103);
        assert_eq!(decoded.1, packet);
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_crc_corruption_detected() {
        let packet = NexaPacket::MouseButton(MouseButtonMsg {
            button: MouseButton::Left,
            is_down: true,
        });

        let mut buffer = BytesMut::new();
        encode_packet(&packet, 104, &mut buffer);

        // Corrompe 1 byte do payload
        buffer[HEADER_SIZE] ^= 0xFF;

        let result = decode_packet(&mut buffer);
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("CRC32"));
    }

    #[test]
    fn test_incomplete_buffer_waits_for_data() {
        let packet = NexaPacket::Ping(Ping {
            timestamp_nanos: 123456789,
        });

        let mut buffer = BytesMut::new();
        encode_packet(&packet, 105, &mut buffer);

        // Trunca propositalmente
        let mut truncated = buffer.split_to(buffer.len() - 3);

        let result = decode_packet(&mut truncated).expect("Não deve dar erro");
        assert_eq!(result, None); // Espera mais dados
    }
}

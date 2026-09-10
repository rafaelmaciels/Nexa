pub mod codec;
pub mod header;
pub mod messages;

pub use codec::{decode_packet, encode_packet, NexaPacket};
pub use header::{MessageType, PacketHeader, HEADER_SIZE, NEXA_MAGIC, PROTOCOL_VERSION};
pub use messages::{
    ClipboardData, KeyEvent, KeyState, MouseButton, MouseButtonMsg, MouseMotionMode, MouseMove,
    MouseWheel, Ping, Pong, ScreenEnter, ScreenLeave,
};

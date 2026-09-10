pub mod clipboard;
pub mod coalescer;
pub mod config;
pub mod engine;
pub mod fsm;
pub mod topology;

pub use clipboard::{ClipboardSyncEngine, DEFAULT_MAX_CLIPBOARD_BYTES};
pub use coalescer::MouseCoalescer;
pub use config::{AppConfig, PeerConfig, DEFAULT_EDGE_DELAY_MS, DEFAULT_PORT};
pub use engine::SessionEngine;
pub use fsm::{SessionFsm, SessionState};
pub use topology::{EdgeSide, ScreenGeometry, ScreenNeighbor, ScreenTopology};


pub mod client;
pub mod connection;
pub mod diagnostic;
pub mod discovery;
pub mod heartbeat;
pub mod interfaces;
pub mod server;

pub use client::NexaClient;
pub use connection::SecureConnection;
pub use diagnostic::{run_connection_diagnostic, ConnectionDiagnostic, DiagnosticStepStatus};
pub use discovery::{DiscoveredDevice, NexaDiscoveryService, DEFAULT_DISCOVERY_PORT};
pub use heartbeat::HeartbeatTracker;
pub use interfaces::{detect_local_interfaces, detect_primary_local_ip, LocalNetworkInterface};
pub use server::NexaServer;


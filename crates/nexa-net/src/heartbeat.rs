use std::time::{Duration, Instant};
use nexa_protocol::{NexaPacket, Ping, Pong};

pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_millis(1500); // 1.5s
pub const MAX_MISSED_PINGS: u32 = 3;

/// Monitor de latência e saúde da conexão através de Heartbeats
pub struct HeartbeatTracker {
    last_ping_sent: Option<Instant>,
    last_pong_received: Option<Instant>,
    missed_pings: u32,
    last_rtt: Option<Duration>,
}

impl Default for HeartbeatTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl HeartbeatTracker {
    pub fn new() -> Self {
        Self {
            last_ping_sent: None,
            last_pong_received: None,
            missed_pings: 0,
            last_rtt: None,
        }
    }

    /// Cria um novo pacote de Ping com timestamp em nanossegundos
    pub fn create_ping(&mut self) -> NexaPacket {
        let now = Instant::now();
        self.last_ping_sent = Some(now);
        self.missed_pings += 1;

        // Representação de nanossegundos arbitrários para eco
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        NexaPacket::Ping(Ping {
            timestamp_nanos: nanos,
        })
    }

    /// Cria um pacote Pong correspondente para responder a um Ping
    pub fn create_pong(ping: &Ping) -> NexaPacket {
        NexaPacket::Pong(Pong {
            original_timestamp_nanos: ping.timestamp_nanos,
        })
    }

    /// Processa o Pong recebido e calcula o RTT
    pub fn handle_pong(&mut self, _pong: &Pong) {
        if let Some(sent) = self.last_ping_sent {
            let rtt = sent.elapsed();
            self.last_rtt = Some(rtt);
            self.missed_pings = 0;
            self.last_pong_received = Some(Instant::now());
        }
    }

    /// Verifica se a conexão deve ser considerada morta/inativa
    pub fn is_dead(&self) -> bool {
        self.missed_pings >= MAX_MISSED_PINGS
    }

    /// Retorna o último Round Trip Time medido em microssegundos
    pub fn last_rtt_micros(&self) -> Option<u128> {
        self.last_rtt.map(|d| d.as_micros())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_heartbeat_rtt_calculation() {
        let mut tracker = HeartbeatTracker::new();

        let ping_pkt = tracker.create_ping();
        let ping_msg = match ping_pkt {
            NexaPacket::Ping(p) => p,
            _ => panic!("Esperado pacote Ping"),
        };

        thread::sleep(Duration::from_millis(5));

        let pong_pkt = HeartbeatTracker::create_pong(&ping_msg);
        let pong_msg = match pong_pkt {
            NexaPacket::Pong(p) => p,
            _ => panic!("Esperado pacote Pong"),
        };

        tracker.handle_pong(&pong_msg);

        assert!(!tracker.is_dead());
        let rtt = tracker.last_rtt_micros().expect("RTT deve estar registrado");
        assert!(rtt >= 4000); // ao menos 4ms
    }

    #[test]
    fn test_heartbeat_timeout_detection() {
        let mut tracker = HeartbeatTracker::new();

        tracker.create_ping();
        tracker.create_ping();
        assert!(!tracker.is_dead());

        tracker.create_ping(); // 3 pings perdidos sem pong
        assert!(tracker.is_dead());
    }
}

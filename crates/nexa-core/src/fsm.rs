use std::time::{Duration, Instant};
use crate::topology::EdgeSide;

/// Estados da Máquina de Estados (FSM) de Controle do Cursor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// O usuário está operando na tela física local
    LocalActive,
    /// O cursor está encostado em uma borda configurada, aguardando confirmação (delay/pressão)
    EdgeTriggered {
        side: EdgeSide,
        target_screen: String,
    },
    /// O cursor e teclado estão ativos na máquina remota (entradas locais suprimidas)
    RemoteActive {
        active_target: String,
    },
    /// Conexão perdida ou desativada
    Disconnected,
}

/// Gerenciador da FSM
pub struct SessionFsm {
    state: SessionState,
    edge_delay: Duration,
    edge_start_time: Option<Instant>,
}

impl SessionFsm {
    pub fn new(edge_delay_ms: u64) -> Self {
        Self {
            state: SessionState::LocalActive,
            edge_delay: Duration::from_millis(edge_delay_ms),
            edge_start_time: None,
        }
    }

    pub fn current_state(&self) -> &SessionState {
        &self.state
    }

    /// Disparado quando o cursor encosta em uma borda com vizinho configurado
    pub fn on_edge_hit(&mut self, side: EdgeSide, target_screen: &str) -> bool {
        match &self.state {
            SessionState::LocalActive => {
                let now = Instant::now();
                self.state = SessionState::EdgeTriggered {
                    side,
                    target_screen: target_screen.to_string(),
                };
                self.edge_start_time = Some(now);

                // Se o delay for zero, transiciona imediatamente
                if self.edge_delay.is_zero() {
                    self.state = SessionState::RemoteActive {
                        active_target: target_screen.to_string(),
                    };
                    self.edge_start_time = None;
                    return true;
                }
                false
            }
            SessionState::EdgeTriggered { side: s, target_screen: t } => {
                if *s == side && t == target_screen {
                    // Checa se o tempo de permanência na borda atingiu o delay
                    if let Some(start) = self.edge_start_time {
                        if start.elapsed() >= self.edge_delay {
                            self.state = SessionState::RemoteActive {
                                active_target: target_screen.to_string(),
                            };
                            self.edge_start_time = None;
                            return true;
                        }
                    }
                }
                false
            }
            SessionState::RemoteActive { .. } | SessionState::Disconnected => false,
        }
    }

    /// Disparado se o cursor se afasta da borda antes de completar o tempo
    pub fn on_edge_abort(&mut self) {
        if matches!(self.state, SessionState::EdgeTriggered { .. }) {
            self.state = SessionState::LocalActive;
            self.edge_start_time = None;
        }
    }

    /// Retorno do cursor para a tela local vindo do nó remoto
    pub fn on_return_to_local(&mut self) {
        self.state = SessionState::LocalActive;
        self.edge_start_time = None;
    }

    /// Notificação de desconexão da rede
    pub fn on_disconnect(&mut self) {
        self.state = SessionState::Disconnected;
        self.edge_start_time = None;
    }

    /// Reconexão / reset
    pub fn on_reconnect(&mut self) {
        self.state = SessionState::LocalActive;
        self.edge_start_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_immediate_switch_when_zero_delay() {
        let mut fsm = SessionFsm::new(0);
        assert_eq!(fsm.current_state(), &SessionState::LocalActive);

        let switched = fsm.on_edge_hit(EdgeSide::Right, "linux_laptop");
        assert!(switched);
        assert_eq!(
            fsm.current_state(),
            &SessionState::RemoteActive {
                active_target: "linux_laptop".into()
            }
        );

        fsm.on_return_to_local();
        assert_eq!(fsm.current_state(), &SessionState::LocalActive);
    }

    #[test]
    fn test_delayed_switch() {
        let mut fsm = SessionFsm::new(50); // 50ms de delay
        assert_eq!(fsm.current_state(), &SessionState::LocalActive);

        // Primeiro toque: não deve trocar imediatamente
        let switched = fsm.on_edge_hit(EdgeSide::Right, "linux_laptop");
        assert!(!switched);
        assert!(matches!(fsm.current_state(), SessionState::EdgeTriggered { .. }));

        // Aguarda 60ms
        thread::sleep(Duration::from_millis(60));

        // Segundo toque após expirar o delay: deve trocar
        let switched2 = fsm.on_edge_hit(EdgeSide::Right, "linux_laptop");
        assert!(switched2);
        assert_eq!(
            fsm.current_state(),
            &SessionState::RemoteActive {
                active_target: "linux_laptop".into()
            }
        );
    }

    #[test]
    fn test_edge_abort() {
        let mut fsm = SessionFsm::new(100);
        fsm.on_edge_hit(EdgeSide::Right, "linux_laptop");
        assert!(matches!(fsm.current_state(), SessionState::EdgeTriggered { .. }));

        fsm.on_edge_abort();
        assert_eq!(fsm.current_state(), &SessionState::LocalActive);
    }
}

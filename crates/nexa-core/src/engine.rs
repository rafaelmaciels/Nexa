use nexa_common::NexaError;
use nexa_platform::{ClipboardManager, InputInjector, ScreenManager};
use nexa_protocol::{
    KeyEvent, KeyState, MouseButton, MouseButtonMsg, MouseMotionMode, MouseMove, MouseWheel,
    NexaPacket, ScreenEnter, ScreenLeave,
};
use crate::clipboard::ClipboardSyncEngine;
use crate::coalescer::MouseCoalescer;
use crate::fsm::{SessionFsm, SessionState};
use crate::topology::{EdgeSide, ScreenGeometry, ScreenTopology};

/// Motor central de orquestração de sessão e despacho de periféricos
pub struct SessionEngine {
    local_screen_id: String,
    local_geometry: ScreenGeometry,
    topology: ScreenTopology,
    fsm: SessionFsm,
    coalescer: MouseCoalescer,
    clipboard_engine: ClipboardSyncEngine,
    last_mouse_x: Option<i32>,
    last_mouse_y: Option<i32>,
    virtual_remote_x: f64,
    virtual_remote_y: f64,
}

impl SessionEngine {
    pub fn new(
        local_screen_id: impl Into<String>,
        local_geometry: ScreenGeometry,
        edge_delay_ms: u64,
    ) -> Self {
        let local_id = local_screen_id.into();
        let mut topology = ScreenTopology::new();
        topology.register_screen(&local_id, local_geometry.clone());

        Self {
            local_screen_id: local_id,
            local_geometry,
            topology,
            fsm: SessionFsm::new(edge_delay_ms),
            coalescer: MouseCoalescer::default(),
            clipboard_engine: ClipboardSyncEngine::default(),
            last_mouse_x: None,
            last_mouse_y: None,
            virtual_remote_x: 0.0,
            virtual_remote_y: 0.0,
        }
    }

    pub fn fsm(&self) -> &SessionFsm {
        &self.fsm
    }

    pub fn fsm_mut(&mut self) -> &mut SessionFsm {
        &mut self.fsm
    }

    pub fn topology_mut(&mut self) -> &mut ScreenTopology {
        &mut self.topology
    }

    pub fn clipboard(&self) -> &ClipboardSyncEngine {
        &self.clipboard_engine
    }

    pub fn clipboard_mut(&mut self) -> &mut ClipboardSyncEngine {
        &mut self.clipboard_engine
    }

    /// Executa sincronização de área de transferência ao alternar telas
    pub fn sync_clipboard_on_transition<C: ClipboardManager>(
        &mut self,
        clip_mgr: &C,
    ) -> Result<Option<NexaPacket>, NexaError> {
        self.clipboard_engine.sync_on_screen_switch(clip_mgr)
    }

    /// Processa payload de clipboard recebido remotamente
    pub fn handle_remote_clipboard<C: ClipboardManager>(
        &mut self,
        text: &str,
        clip_mgr: &C,
    ) -> Result<bool, NexaError> {
        self.clipboard_engine.on_remote_clipboard_received(text, clip_mgr)
    }

    /// Processa movimento físico capturado na máquina local
    /// Retorna `Some(packet)` quando há evento para despacho remoto (como ScreenEnter ou MouseMove)
    pub fn handle_local_mouse_move(
        &mut self,
        x: i32,
        y: i32,
        is_relative: bool,
    ) -> Result<Option<NexaPacket>, NexaError> {
        match self.fsm.current_state() {
            SessionState::LocalActive | SessionState::EdgeTriggered { .. } => {
                // Checa aproximação contra as bordas da tela física local
                let right_edge = self.local_geometry.width as i32 - 1;
                let bottom_edge = self.local_geometry.height as i32 - 1;

                // Margem de tolerância de 4 pixels para garantir que o cursor toque a borda com precisão
                let hit_side = if x >= right_edge - 3 {
                    Some(EdgeSide::Right)
                } else if x <= 3 {
                    Some(EdgeSide::Left)
                } else if y >= bottom_edge - 3 {
                    Some(EdgeSide::Bottom)
                } else if y <= 3 {
                    Some(EdgeSide::Top)
                } else {
                    None
                };

                if let Some(side) = hit_side {
                    if let Some(neighbor) = self.topology.get_neighbor(&self.local_screen_id, side) {
                        let target_id = neighbor.target_screen_id.clone();
                        let transitioned = self.fsm.on_edge_hit(side, &target_id);

                        if transitioned {
                            self.virtual_remote_x = if side == EdgeSide::Right { 0.0 } else { 1920.0 };
                            self.virtual_remote_y = y as f64;
                            self.last_mouse_x = Some(x);
                            self.last_mouse_y = Some(y);

                            // Calcula entrada proporcional na tela do vizinho
                            let norm_x = match side {
                                EdgeSide::Right => 0u16,
                                EdgeSide::Left => 65535u16,
                                _ => 32768u16,
                            };
                            let norm_y = ((y as f64 / self.local_geometry.height as f64) * 65535.0)
                                .clamp(0.0, 65535.0) as u16;

                            return Ok(Some(NexaPacket::ScreenEnter(ScreenEnter {
                                normalized_x: norm_x,
                                normalized_y: norm_y,
                                lock_keys_mask: 0,
                            })));
                        }
                    }
                } else if x > 15 && x < right_edge - 15 && y > 15 && y < bottom_edge - 15 {
                    self.fsm.on_edge_abort();
                }

                Ok(None)
            }
            SessionState::RemoteActive { .. } => {
                let (dx, dy) = if is_relative {
                    (x, y)
                } else {
                    let prev_x = self.last_mouse_x.unwrap_or(x);
                    let prev_y = self.last_mouse_y.unwrap_or(y);
                    self.last_mouse_x = Some(x);
                    self.last_mouse_y = Some(y);
                    (x - prev_x, y - prev_y)
                };

                self.virtual_remote_x += dx as f64;
                self.virtual_remote_y += dy as f64;

                // Se o cursor foi movimentado de volta para a esquerda da tela remota (x <= 0),
                // o controle do mouse e teclado retorna automaticamente para o computador local (Windows)!
                if self.virtual_remote_x < 0.0 || (dx < -15 && self.virtual_remote_x < 40.0) {
                    self.fsm.on_return_to_local();
                    self.last_mouse_x = None;
                    self.last_mouse_y = None;
                    self.virtual_remote_x = 0.0;
                    return Ok(Some(NexaPacket::ScreenLeave(ScreenLeave { timestamp_ms: 0 })));
                }

                let pkt = MouseMove {
                    mode: MouseMotionMode::Relative,
                    x: dx,
                    y: dy,
                };
                self.coalescer.push(pkt);

                if self.coalescer.should_flush() {
                    Ok(self.coalescer.flush().map(NexaPacket::MouseMove))
                } else {
                    Ok(self.coalescer.flush().map(NexaPacket::MouseMove))
                }
            }
            SessionState::Disconnected => Ok(None),
        }
    }

    /// Processa clique local de mouse
    pub fn handle_local_mouse_button(
        &self,
        button: MouseButton,
        is_down: bool,
    ) -> Option<NexaPacket> {
        if matches!(self.fsm.current_state(), SessionState::RemoteActive { .. }) {
            Some(NexaPacket::MouseButton(MouseButtonMsg {
                button,
                is_down,
            }))
        } else {
            None
        }
    }

    /// Processa rolagem de roda do mouse
    pub fn handle_local_mouse_wheel(&self, delta_x: i16, delta_y: i16) -> Option<NexaPacket> {
        if matches!(self.fsm.current_state(), SessionState::RemoteActive { .. }) {
            Some(NexaPacket::MouseWheel(MouseWheel {
                delta_x,
                delta_y,
                is_high_res: false,
            }))
        } else {
            None
        }
    }

    /// Processa pressionamento ou soltura de tecla
    pub fn handle_local_key(
        &self,
        scancode: u16,
        state: KeyState,
        modifiers: u16,
    ) -> Option<NexaPacket> {
        if matches!(self.fsm.current_state(), SessionState::RemoteActive { .. }) {
            Some(NexaPacket::KeyEvent(KeyEvent {
                scancode,
                state,
                modifiers,
            }))
        } else {
            None
        }
    }

    /// Processa pacotes recebidos pela rede de um nó remoto e despacha para os periféricos locais
    pub fn handle_remote_packet<I: InputInjector, S: ScreenManager>(
        &mut self,
        packet: &NexaPacket,
        injector: &I,
        screen_mgr: &S,
    ) -> Result<Option<NexaPacket>, NexaError> {
        match packet {
            NexaPacket::ScreenEnter(enter) => {
                // Calcula coordenada física a partir dos valores normalizados (0..65535)
                let (_, _, vw, vh) = screen_mgr.get_screen_bounds()?;
                let target_x = ((enter.normalized_x as f64 / 65535.0) * vw as f64).round() as i32;
                let target_y = ((enter.normalized_y as f64 / 65535.0) * vh as f64).round() as i32;

                screen_mgr.set_cursor_position(target_x, target_y)?;
                Ok(None)
            }
            NexaPacket::MouseMove(m) => {
                let is_rel = m.mode == MouseMotionMode::Relative;
                injector.inject_mouse_move(m.x, m.y, is_rel)?;
                Ok(None)
            }
            NexaPacket::MouseButton(b) => {
                injector.inject_mouse_button(b.button, b.is_down)?;
                Ok(None)
            }
            NexaPacket::MouseWheel(w) => {
                injector.inject_mouse_wheel(w.delta_x, w.delta_y)?;
                Ok(None)
            }
            NexaPacket::KeyEvent(k) => {
                injector.inject_key(k.scancode, k.state)?;
                Ok(None)
            }
            NexaPacket::ScreenLeave(_) => {
                self.fsm.on_return_to_local();
                screen_mgr.unclip_cursor()?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Processa pacotes recebidos pela rede com suporte explícito a injeção na área de transferência
    pub fn handle_remote_packet_with_clipboard<I: InputInjector, S: ScreenManager, C: ClipboardManager>(
        &mut self,
        packet: &NexaPacket,
        injector: &I,
        screen_mgr: &S,
        clip_mgr: &C,
    ) -> Result<Option<NexaPacket>, NexaError> {
        if let NexaPacket::Clipboard(clip_data) = packet {
            self.clipboard_engine.on_remote_clipboard_received(&clip_data.text, clip_mgr)?;
            return Ok(None);
        }

        self.handle_remote_packet(packet, injector, screen_mgr)
    }
}

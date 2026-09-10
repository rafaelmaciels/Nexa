use std::time::{Duration, Instant};
use nexa_protocol::{MouseMotionMode, MouseMove};

/// Coalescedor de eventos de movimento de mouse para redução de latência e tráfego de rede
pub struct MouseCoalescer {
    accumulated_dx: i32,
    accumulated_dy: i32,
    last_absolute_x: Option<i32>,
    last_absolute_y: Option<i32>,
    current_mode: Option<MouseMotionMode>,
    last_event_time: Option<Instant>,
    window: Duration,
}

impl Default for MouseCoalescer {
    fn default() -> Self {
        Self::new(Duration::from_millis(3))
    }
}

impl MouseCoalescer {
    pub fn new(window: Duration) -> Self {
        Self {
            accumulated_dx: 0,
            accumulated_dy: 0,
            last_absolute_x: None,
            last_absolute_y: None,
            current_mode: None,
            last_event_time: None,
            window,
        }
    }

    /// Adiciona um novo evento de movimento ao coalescedor
    pub fn push(&mut self, move_event: MouseMove) {
        let now = Instant::now();
        self.last_event_time = Some(now);
        self.current_mode = Some(move_event.mode);

        match move_event.mode {
            MouseMotionMode::Relative => {
                self.accumulated_dx += move_event.x;
                self.accumulated_dy += move_event.y;
            }
            MouseMotionMode::Absolute => {
                self.last_absolute_x = Some(move_event.x);
                self.last_absolute_y = Some(move_event.y);
            }
        }
    }

    /// Verifica se a janela de tempo expirou e o evento consolidado deve ser despachado
    pub fn should_flush(&self) -> bool {
        if let Some(t) = self.last_event_time {
            t.elapsed() >= self.window
        } else {
            false
        }
    }

    /// Descarrega os movimentos acumulados em um único evento consolidado
    pub fn flush(&mut self) -> Option<MouseMove> {
        let mode = self.current_mode.take()?;
        self.last_event_time = None;

        match mode {
            MouseMotionMode::Relative => {
                if self.accumulated_dx == 0 && self.accumulated_dy == 0 {
                    return None;
                }
                let dx = self.accumulated_dx;
                let dy = self.accumulated_dy;
                self.accumulated_dx = 0;
                self.accumulated_dy = 0;

                Some(MouseMove {
                    mode: MouseMotionMode::Relative,
                    x: dx,
                    y: dy,
                })
            }
            MouseMotionMode::Absolute => {
                let x = self.last_absolute_x.take()?;
                let y = self.last_absolute_y.take()?;

                Some(MouseMove {
                    mode: MouseMotionMode::Absolute,
                    x,
                    y,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_mouse_coalescing() {
        let mut coalescer = MouseCoalescer::new(Duration::from_millis(5));

        // Simula 5 eventos rápidos de mouse a 1000 Hz
        coalescer.push(MouseMove {
            mode: MouseMotionMode::Relative,
            x: 2,
            y: 3,
        });
        coalescer.push(MouseMove {
            mode: MouseMotionMode::Relative,
            x: 4,
            y: -1,
        });
        coalescer.push(MouseMove {
            mode: MouseMotionMode::Relative,
            x: 1,
            y: 2,
        });

        // O flush deve retornar um único evento consolidado com a soma dos deltas
        let consolidated = coalescer.flush().expect("Deve consolidar");
        assert_eq!(consolidated.mode, MouseMotionMode::Relative);
        assert_eq!(consolidated.x, 7); // 2 + 4 + 1 = 7
        assert_eq!(consolidated.y, 4); // 3 - 1 + 2 = 4

        // Segundo flush consecutivo deve ser None (sem dados pendentes)
        assert!(coalescer.flush().is_none());
    }

    #[test]
    fn test_absolute_mouse_coalescing() {
        let mut coalescer = MouseCoalescer::new(Duration::from_millis(5));

        // Simula posições absolutas consecutivas
        coalescer.push(MouseMove {
            mode: MouseMotionMode::Absolute,
            x: 100,
            y: 200,
        });
        coalescer.push(MouseMove {
            mode: MouseMotionMode::Absolute,
            x: 110,
            y: 215,
        });

        // Deve retornar a última posição absoluta mais recente
        let consolidated = coalescer.flush().expect("Deve consolidar");
        assert_eq!(consolidated.mode, MouseMotionMode::Absolute);
        assert_eq!(consolidated.x, 110);
        assert_eq!(consolidated.y, 215);
    }
}

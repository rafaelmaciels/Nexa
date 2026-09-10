use std::collections::HashMap;

/// Lado da borda da tela
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Informações de geometria de uma tela
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ScreenGeometry {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    pub fn with_origin(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Conexão de adjacência entre telas
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenNeighbor {
    pub target_screen_id: String,
    pub target_side: EdgeSide,
}

/// Gerenciador de Topologia Espacial de Telas
#[derive(Debug, Clone, Default)]
pub struct ScreenTopology {
    screens: HashMap<String, ScreenGeometry>,
    // screen_id -> (EdgeSide -> ScreenNeighbor)
    links: HashMap<String, HashMap<EdgeSide, ScreenNeighbor>>,
}

impl ScreenTopology {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra uma tela na topologia
    pub fn register_screen(&mut self, id: impl Into<String>, geometry: ScreenGeometry) {
        self.screens.insert(id.into(), geometry);
    }

    /// Conecta duas telas bidirecionalmente (ex: tela_a à esquerda de tela_b)
    pub fn link_horizontal(&mut self, left_id: &str, right_id: &str) {
        self.link_directed(left_id, EdgeSide::Right, right_id, EdgeSide::Left);
        self.link_directed(right_id, EdgeSide::Left, left_id, EdgeSide::Right);
    }

    /// Conecta duas telas bidirecionalmente na vertical (ex: tela_a acima de tela_b)
    pub fn link_vertical(&mut self, top_id: &str, bottom_id: &str) {
        self.link_directed(top_id, EdgeSide::Bottom, bottom_id, EdgeSide::Top);
        self.link_directed(bottom_id, EdgeSide::Top, top_id, EdgeSide::Bottom);
    }

    /// Conexão direcionada
    pub fn link_directed(
        &mut self,
        from_id: &str,
        from_side: EdgeSide,
        to_id: &str,
        to_side: EdgeSide,
    ) {
        let entry = self.links.entry(from_id.to_string()).or_default();
        entry.insert(
            from_side,
            ScreenNeighbor {
                target_screen_id: to_id.to_string(),
                target_side: to_side,
            },
        );
    }

    /// Obtém o vizinho em uma borda específica
    pub fn get_neighbor(&self, screen_id: &str, side: EdgeSide) -> Option<&ScreenNeighbor> {
        self.links.get(screen_id).and_then(|m| m.get(&side))
    }

    /// Calcula o ponto de entrada proporcional no monitor remoto
    /// Retorna `(target_screen_id, target_x, target_y)`
    pub fn calculate_transition(
        &self,
        current_screen_id: &str,
        exit_side: EdgeSide,
        exit_x: i32,
        exit_y: i32,
    ) -> Option<(String, i32, i32)> {
        let current_geo = self.screens.get(current_screen_id)?;
        let neighbor = self.get_neighbor(current_screen_id, exit_side)?;
        let target_geo = self.screens.get(&neighbor.target_screen_id)?;

        let (target_x, target_y) = match exit_side {
            EdgeSide::Right => {
                // Saindo pela direita: entra pela esquerda do vizinho mantendo a proporção vertical Y
                let ratio_y = (exit_y as f64) / (current_geo.height as f64).max(1.0);
                let target_y = (ratio_y * target_geo.height as f64).round() as i32;
                (0, target_y.clamp(0, target_geo.height as i32 - 1))
            }
            EdgeSide::Left => {
                // Saindo pela esquerda: entra pela direita do vizinho mantendo proporção Y
                let ratio_y = (exit_y as f64) / (current_geo.height as f64).max(1.0);
                let target_y = (ratio_y * target_geo.height as f64).round() as i32;
                (
                    target_geo.width as i32 - 1,
                    target_y.clamp(0, target_geo.height as i32 - 1),
                )
            }
            EdgeSide::Bottom => {
                // Saindo por baixo: entra por cima mantendo proporção X
                let ratio_x = (exit_x as f64) / (current_geo.width as f64).max(1.0);
                let target_x = (ratio_x * target_geo.width as f64).round() as i32;
                (target_x.clamp(0, target_geo.width as i32 - 1), 0)
            }
            EdgeSide::Top => {
                // Saindo por cima: entra por baixo mantendo proporção X
                let ratio_x = (exit_x as f64) / (current_geo.width as f64).max(1.0);
                let target_x = (ratio_x * target_geo.width as f64).round() as i32;
                (
                    target_x.clamp(0, target_geo.width as i32 - 1),
                    target_geo.height as i32 - 1,
                )
            }
        };

        Some((neighbor.target_screen_id.clone(), target_x, target_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_screen_transition() {
        let mut topology = ScreenTopology::new();

        // Windows 11 (1920x1080)
        topology.register_screen("windows_pc", ScreenGeometry::new(1920, 1080));
        // elementary OS 8.1 (2560x1440)
        topology.register_screen("linux_laptop", ScreenGeometry::new(2560, 1440));

        // Windows à esquerda, Linux à direita
        topology.link_horizontal("windows_pc", "linux_laptop");

        // Cursor sai na borda direita do Windows a 50% da altura (Y=540)
        let (target, target_x, target_y) = topology
            .calculate_transition("windows_pc", EdgeSide::Right, 1919, 540)
            .expect("Transição esperada");

        assert_eq!(target, "linux_laptop");
        assert_eq!(target_x, 0); // Entra na borda esquerda
        assert_eq!(target_y, 720); // 50% de 1440 = 720

        // Retorno: sai na borda esquerda do Linux a 720
        let (target_back, back_x, back_y) = topology
            .calculate_transition("linux_laptop", EdgeSide::Left, 0, 720)
            .expect("Retorno esperado");

        assert_eq!(target_back, "windows_pc");
        assert_eq!(back_x, 1919); // Entra na borda direita
        assert_eq!(back_y, 540); // 50% de 1080 = 540
    }
}

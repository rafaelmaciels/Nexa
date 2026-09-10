use std::collections::HashMap;
use nexa_core::{EdgeSide, ScreenTopology};

/// Posição visual relativa de um computador em relação a outro na tela de configuração
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenPosition {
    Left,
    Right,
    Above,
    Below,
}

impl ScreenPosition {
    pub fn to_edge_side(&self) -> EdgeSide {
        match self {
            ScreenPosition::Left => EdgeSide::Left,
            ScreenPosition::Right => EdgeSide::Right,
            ScreenPosition::Above => EdgeSide::Top,
            ScreenPosition::Below => EdgeSide::Bottom,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            ScreenPosition::Left => ScreenPosition::Right,
            ScreenPosition::Right => ScreenPosition::Left,
            ScreenPosition::Above => ScreenPosition::Below,
            ScreenPosition::Below => ScreenPosition::Above,
        }
    }
}

/// Gerenciador de layout visual 2D configurado pelo usuário
#[derive(Default)]
pub struct VisualLayoutManager {
    // from_screen -> (ScreenPosition, to_screen)
    links: HashMap<String, Vec<(ScreenPosition, String)>>,
}

impl VisualLayoutManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Posiciona uma tela em relação a outra (ex: colocar 'Linux' à direita de 'Windows')
    pub fn place_screen(
        &mut self,
        target_screen: impl Into<String>,
        position: ScreenPosition,
        relative_to: impl Into<String>,
    ) {
        let target = target_screen.into();
        let base = relative_to.into();

        // Conecta base -> position -> target
        let entry_base = self.links.entry(base.clone()).or_default();
        entry_base.retain(|(pos, _)| *pos != position);
        entry_base.push((position, target.clone()));

        // Conecta reversamente target -> opposite -> base
        let entry_target = self.links.entry(target).or_default();
        let opposite = position.opposite();
        entry_target.retain(|(pos, _)| *pos != opposite);
        entry_target.push((opposite, base));
    }

    /// Sincroniza o layout visual 2D com a ScreenTopology do motor central
    pub fn sync_to_topology(&self, topology: &mut ScreenTopology) {
        for (from, targets) in &self.links {
            for (pos, to) in targets {
                let from_edge = pos.to_edge_side();
                let to_edge = pos.opposite().to_edge_side();
                topology.link_directed(from, from_edge, to, to_edge);
            }
        }
    }

    pub fn get_placements(&self, screen_id: &str) -> Option<&Vec<(ScreenPosition, String)>> {
        self.links.get(screen_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexa_core::ScreenGeometry;

    #[test]
    fn test_visual_layout_sync_to_topology() {
        let mut layout = VisualLayoutManager::new();

        // Posiciona Linux à direita do Windows
        layout.place_screen("Linux-Laptop", ScreenPosition::Right, "Windows-PC");

        // Posiciona Servidor abaixo do Windows
        layout.place_screen("Servidor", ScreenPosition::Below, "Windows-PC");

        let mut topology = ScreenTopology::new();
        topology.register_screen("Windows-PC", ScreenGeometry::new(1920, 1080));
        topology.register_screen("Linux-Laptop", ScreenGeometry::new(2560, 1440));
        topology.register_screen("Servidor", ScreenGeometry::new(1920, 1080));

        layout.sync_to_topology(&mut topology);

        // Valida que ao sair pela direita no Windows, o vizinho é o Linux-Laptop
        let right_neighbor = topology.get_neighbor("Windows-PC", EdgeSide::Right).unwrap();
        assert_eq!(right_neighbor.target_screen_id, "Linux-Laptop");
        assert_eq!(right_neighbor.target_side, EdgeSide::Left);

        // Valida que ao sair por baixo no Windows, o vizinho é o Servidor
        let bottom_neighbor = topology.get_neighbor("Windows-PC", EdgeSide::Bottom).unwrap();
        assert_eq!(bottom_neighbor.target_screen_id, "Servidor");
        assert_eq!(bottom_neighbor.target_side, EdgeSide::Top);

        // Valida que o Linux também aponta de volta para a esquerda para o Windows
        let left_neighbor = topology.get_neighbor("Linux-Laptop", EdgeSide::Left).unwrap();
        assert_eq!(left_neighbor.target_screen_id, "Windows-PC");
    }
}

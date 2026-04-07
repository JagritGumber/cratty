#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    Proportion(f32),
    Fixed(f32),
}

impl ColumnWidth {
    pub fn resolve(self, viewport_w: f32) -> f32 {
        match self {
            Self::Proportion(frac) => viewport_w * frac,
            Self::Fixed(px) => px,
        }
    }

    pub fn adjust(self, delta: f32) -> Self {
        match self {
            Self::Proportion(p) => Self::Proportion((p + delta).clamp(0.1, 1.0)),
            Self::Fixed(px) => Self::Fixed((px + delta * 1000.0).max(100.0)),
        }
    }

    pub fn same_as(self, other: Self) -> bool {
        match (self, other) {
            (Self::Proportion(x), Self::Proportion(y)) => (x - y).abs() < 0.01,
            (Self::Fixed(x), Self::Fixed(y)) => (x - y).abs() < 0.5,
            _ => false,
        }
    }
}

pub fn default_presets() -> Vec<ColumnWidth> {
    vec![
        ColumnWidth::Proportion(0.333),
        ColumnWidth::Proportion(0.5),
        ColumnWidth::Proportion(0.667),
    ]
}

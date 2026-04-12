/// Current viewport scroll state for a paper strip.
#[derive(Debug, Clone, Copy)]
pub enum ViewOffset {
    /// Resting at a fixed position.
    Static(f32),
    /// Animating between two positions.
    Animating { from: f32, to: f32, progress: f32 },
}

impl Default for ViewOffset {
    fn default() -> Self {
        Self::Static(0.0)
    }
}

impl ViewOffset {
    /// Current interpolated position.
    pub fn current(&self) -> f32 {
        match self {
            Self::Static(x) => *x,
            Self::Animating { from, to, progress } => {
                let t = ease_out_quart(*progress);
                from + (to - from) * t
            }
        }
    }

    /// Advance animation by `dt` (0.0..1.0 step). Returns true if still animating.
    pub fn tick(&mut self, dt: f32) -> bool {
        if let Self::Animating { to, progress, .. } = self {
            *progress = (*progress + dt).min(1.0);
            if *progress >= 1.0 {
                *self = Self::Static(*to);
                return false;
            }
            return true;
        }
        false
    }

    pub fn is_animating(&self) -> bool { matches!(self, Self::Animating { .. }) }

    /// Start animating to a new target.
    pub fn animate_to(&mut self, target: f32) {
        let current = self.current();
        if (current - target).abs() < 0.5 {
            *self = Self::Static(target);
        } else {
            *self = Self::Animating { from: current, to: target, progress: 0.0 };
        }
    }
}

pub fn ease_out_quart(t: f32) -> f32 {
    let inv = 1.0 - t;
    let inv2 = inv * inv;
    1.0 - inv2 * inv2
}

/// Pixel dimensions derived from the configured font size.
///
/// The caller computes `cell_w` and `cell_h` once at startup (and again
/// on any font-size change), then threads this struct through the
/// rendering pipeline so no module needs to reach for a global constant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    pub font_size: f32,
    pub cell_w: f32,
    pub cell_h: f32,
}

impl FontMetrics {
    /// Build metrics from a font size using the standard monospace
    /// heuristic: `cell_w ~ 0.6 * size`, `cell_h ~ 1.3 * size`
    /// (rounded to the nearest integer pixel).
    pub fn from_size(font_size: f32) -> Self {
        let cell_w = (font_size * 0.6).round();
        let cell_h = (font_size * 1.3).round();
        Self { font_size, cell_w, cell_h }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_14pt() {
        let m = FontMetrics::from_size(14.0);
        assert_eq!(m.cell_w, 8.0);
        assert_eq!(m.cell_h, 18.0);
    }

    #[test]
    fn larger_size() {
        let m = FontMetrics::from_size(20.0);
        assert_eq!(m.cell_w, 12.0);
        assert_eq!(m.cell_h, 26.0);
    }
}

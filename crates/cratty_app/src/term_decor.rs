use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Color, Point};

/// Draw a single-pixel underline at the bottom of a cell.
pub fn draw_underline(
    frame: &mut Frame, origin: Point, color: Color,
    cell_w: f32, cell_h: f32,
) {
    let y = origin.y + cell_h - 2.0;
    let stroke = Stroke::default().with_color(color).with_width(1.0);
    let path = Path::line(
        Point::new(origin.x, y),
        Point::new(origin.x + cell_w, y),
    );
    frame.stroke(&path, stroke);
}

/// Draw a single-pixel strikethrough at the vertical center of a cell.
pub fn draw_strikethrough(
    frame: &mut Frame, origin: Point, color: Color,
    cell_w: f32, cell_h: f32,
) {
    let y = origin.y + cell_h / 2.0;
    let stroke = Stroke::default().with_color(color).with_width(1.0);
    let path = Path::line(
        Point::new(origin.x, y),
        Point::new(origin.x + cell_w, y),
    );
    frame.stroke(&path, stroke);
}

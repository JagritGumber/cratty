use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Color, Point, Size};

const CURSOR_COLOR: Color = Color::from_rgba(0.686, 0.675, 0.725, 0.7); // #afacb9

/// Draw the terminal cursor at the given position.
/// Focused panes get a filled block; unfocused panes get a hollow outline.
pub fn draw_cursor(
    frame: &mut Frame, point: Point, focused: bool,
    cell_w: f32, cell_h: f32,
) {
    let size = Size::new(cell_w, cell_h);
    if focused {
        frame.fill_rectangle(point, size, CURSOR_COLOR);
    } else {
        let stroke = Stroke::default()
            .with_color(CURSOR_COLOR)
            .with_width(1.0);
        let path = Path::rectangle(point, size);
        frame.stroke(&path, stroke);
    }
}

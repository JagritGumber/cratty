use std::collections::HashMap;
use iced::widget::{container, row, scrollable, text, Id, Space};
use iced::{Color, Element, Length};
use cratty_core::{FontMetrics, PaneId, PaperStrip};
use crate::message::Message;
use crate::pane::Pane;
use crate::style::FG_MUTED;

pub fn strip_scroll_id() -> Id { Id::new("paper-strip") }

pub fn view_strip<'a>(
    strip: &PaperStrip, panes: &'a HashMap<PaneId, Pane>,
    viewport_w: f32, metrics: FontMetrics,
) -> Element<'a, Message> {
    if strip.panes.is_empty() {
        return container(Space::new()).width(Length::Fill).height(Length::Fill).into();
    }
    let mut elements: Vec<Element<Message>> = Vec::new();
    for (i, pane_id) in strip.panes.iter().enumerate() {
        if i > 0 { elements.push(pane_divider()); }
        if let Some(pane) = panes.get(pane_id) {
            let pane_w = strip.pane_width_rendered(i, viewport_w);
            let distance = if i == strip.focus_idx { 0.0 } else { 1.0 };
            elements.push(render_pane(pane, distance, pane_w, metrics));
        }
    }

    let strip = scrollable(row(elements).spacing(0).height(Length::Fill))
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().width(0).scroller_width(0),
        ))
        .id(strip_scroll_id())
        .width(Length::Fill)
        .height(Length::Fill);

    container(strip).width(Length::Fill).height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(crate::style::BG_TITLEBAR)),
            ..Default::default()
        })
        .into()
}

fn render_pane(pane: &Pane, distance: f32, pane_w: f32, metrics: FontMetrics) -> Element<'_, Message> {
    let focused_color = crate::style::ACCENT;
    let unfocused_color = FG_MUTED;
    let border_color = lerp_color(focused_color, unfocused_color, distance);
    let border_w = 2.0 - distance;

    let focused = distance < 0.01;
    let inner: Element<Message> = match &pane.backend {
        Some(b) => crate::term_widget::view(b, focused, metrics),
        None => container(text("Loading...").size(12).color(crate::style::FG_DIM))
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(crate::style::BG_TERMINAL)),
                ..Default::default()
            }).into(),
    };

    let term_view = container(inner)
        .width(Length::Fixed(pane_w)).height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(crate::style::BG_TERMINAL)),
            border: iced::Border {
                color: border_color, width: border_w, radius: 2.0.into(),
            },
            ..Default::default()
        });

    if distance < 0.01 {
        term_view.into()
    } else {
        let overlay_alpha = distance * 0.35;
        iced::widget::stack![
            term_view,
            container(Space::new())
                .width(Length::Fixed(pane_w)).height(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(
                        Color::from_rgba(0.0, 0.0, 0.0, overlay_alpha),
                    )),
                    ..Default::default()
                }),
        ]
        .width(Length::Fixed(pane_w)).height(Length::Fill).into()
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgb(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t)
}

fn pane_divider() -> Element<'static, Message> {
    container(Space::new()).width(2).height(Length::Fill).style(|_| container::Style {
        background: Some(iced::Background::Color(FG_MUTED)), ..Default::default()
    }).into()
}

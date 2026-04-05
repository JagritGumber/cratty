use std::collections::HashMap;

use iced::widget::{container, row, text, Space};
use iced::{Color, Element, Length};

use cratty_core::{PaneId, PaperStrip};

use crate::message::Message;
use crate::pane::Pane;
use crate::style::FG_MUTED;

/// Render the paper strip: focused pane + adjacent panes side by side.
pub fn view_strip<'a>(
    strip: &PaperStrip, panes: &'a HashMap<PaneId, Pane>,
) -> Element<'a, Message> {
    if strip.panes.is_empty() {
        return container(Space::new()).width(Length::Fill).height(Length::Fill).into();
    }

    let focus = strip.focus_idx;
    let mut elements: Vec<Element<Message>> = Vec::new();

    for (i, idx) in visible_range(focus, strip.panes.len()).iter().enumerate() {
        if i > 0 { elements.push(pane_divider()); }
        if let Some(pane) = panes.get(&strip.panes[*idx]) {
            elements.push(render_pane(pane, *idx == focus));
        }
    }

    row(elements).spacing(0).width(Length::Fill).height(Length::Fill).into()
}

fn visible_range(focus: usize, count: usize) -> Vec<usize> {
    let mut r = Vec::new();
    if focus > 0 { r.push(focus - 1); }
    r.push(focus);
    if focus + 1 < count { r.push(focus + 1); }
    r
}

fn render_pane(pane: &Pane, is_focused: bool) -> Element<'_, Message> {
    let border_color = if is_focused {
        Color::from_rgb(0.30, 0.65, 0.90)
    } else {
        Color::from_rgb(0.15, 0.15, 0.15)
    };
    let border_w = if is_focused { 2.0 } else { 1.0 };

    let inner: Element<Message> = match &pane.backend {
        Some(backend) => crate::term_widget::view(backend),
        None => container(text("Loading...").size(12).color(crate::style::FG_DIM))
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(crate::style::BG_TERMINAL)),
                ..Default::default()
            })
            .into(),
    };
    let term_view = container(inner)
        .width(Length::FillPortion(1)).height(Length::Fill)
        .style(move |_| container::Style {
            border: iced::Border { color: border_color, width: border_w, radius: 0.0.into() },
            ..Default::default()
        });

    if is_focused {
        term_view.into()
    } else {
        iced::widget::stack![
            term_view,
            container(Space::new()).width(Length::Fill).height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.35))),
                    ..Default::default()
                }),
        ]
        .width(Length::FillPortion(1)).height(Length::Fill).into()
    }
}

fn pane_divider() -> Element<'static, Message> {
    container(Space::new()).width(2).height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(FG_MUTED)),
            ..Default::default()
        })
        .into()
}

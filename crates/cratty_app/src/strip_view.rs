use std::collections::HashMap;

use iced::widget::{container, row, Space};
use iced::{Color, Element, Length};

use cratty_core::{PaneId, PaperStrip};

use crate::message::Message;
use crate::pane::Pane;
use crate::style::FG_MUTED;

/// Render the paper strip. Shows focused pane and adjacent panes using
/// FillPortion layout (no scrollable -- iced_term can't render in one).
pub fn view_strip<'a>(
    strip: &PaperStrip, panes: &'a HashMap<PaneId, Pane>,
) -> Element<'a, Message> {
    let focus = strip.focus_idx;
    let count = strip.panes.len();

    if count == 0 {
        return container(Space::new())
            .width(Length::Fill).height(Length::Fill).into();
    }

    // Single pane: fill the whole area
    if count == 1 {
        if let Some(pane) = strip.panes.first().and_then(|pid| panes.get(pid)) {
            return render_single_pane(pane);
        }
    }

    // Multiple panes: show focused + adjacent using FillPortion
    let mut elements: Vec<Element<Message>> = Vec::new();
    let visible = visible_range(focus, count);

    for (i, idx) in visible.iter().enumerate() {
        if i > 0 {
            elements.push(pane_divider());
        }
        let pid = strip.panes[*idx];
        if let Some(pane) = panes.get(&pid) {
            let is_focused = *idx == focus;
            elements.push(render_pane(pane, is_focused));
        }
    }

    row(elements).spacing(0).width(Length::Fill).height(Length::Fill).into()
}

/// Which pane indices to show. Focused pane + up to one on each side.
fn visible_range(focus: usize, count: usize) -> Vec<usize> {
    let mut result = Vec::new();
    if focus > 0 {
        result.push(focus - 1);
    }
    result.push(focus);
    if focus + 1 < count {
        result.push(focus + 1);
    }
    result
}

fn render_single_pane(pane: &Pane) -> Element<'_, Message> {
    iced::widget::keyed_column(std::iter::once((
        pane.term_id,
        container(
            iced_term::TerminalView::show(&pane.terminal).map(Message::TermEvent),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn render_pane(pane: &Pane, is_focused: bool) -> Element<'_, Message> {
    let border_color = if is_focused {
        Color::from_rgb(0.30, 0.65, 0.90)
    } else {
        Color::from_rgb(0.15, 0.15, 0.15)
    };
    let border_width = if is_focused { 2.0 } else { 1.0 };

    let term_view = container(
        iced_term::TerminalView::show(&pane.terminal).map(Message::TermEvent),
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .style(move |_| container::Style {
        border: iced::Border {
            color: border_color,
            width: border_width,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    if is_focused {
        term_view.into()
    } else {
        // Dim overlay on unfocused panes
        iced::widget::stack![
            term_view,
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(
                        Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                    )),
                    ..Default::default()
                }),
        ]
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .into()
    }
}

fn pane_divider() -> Element<'static, Message> {
    container(Space::new())
        .width(2)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(FG_MUTED)),
            ..Default::default()
        })
        .into()
}

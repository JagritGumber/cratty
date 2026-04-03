use std::collections::HashMap;

use iced::widget::container;
use iced::{Element, Length};

use cratty_core::{PaneId, PaperStrip};

use crate::message::Message;
use crate::pane::Pane;

/// Render the focused pane of a paper strip.
/// TODO: render adjacent panes with horizontal scrolling once iced_term
/// supports fixed-width rendering inside scrollable containers.
pub fn view_strip<'a>(
    strip: &PaperStrip, panes: &'a HashMap<PaneId, Pane>,
) -> Element<'a, Message> {
    let focused = strip.focused_pane().and_then(|pid| panes.get(&pid));

    match focused {
        Some(pane) => {
            iced::widget::keyed_column(std::iter::once((
                pane.term_id,
                container(
                    iced_term::TerminalView::show(&pane.terminal)
                        .map(Message::TermEvent),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            )))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
        None => {
            container(iced::widget::Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}

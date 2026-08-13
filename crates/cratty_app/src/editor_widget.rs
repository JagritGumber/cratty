use iced::widget::{column, container, text, text_editor, Id};
use iced::{Element, Font, Length};

use crate::message::Message;
use crate::pane::CodePane;
use crate::style::{BG_TERMINAL, FG_ACTIVE};
use crate::widgets::tooltip_surface_style;
use cratty_core::{FontMetrics, PaneId};

pub fn editor_id(pane_id: PaneId) -> Id {
    Id::from(format!("code-editor-{}", pane_id.0))
}

pub fn view<'a>(
    pane_id: PaneId, code: &'a CodePane, metrics: FontMetrics,
) -> Element<'a, Message> {
    let editor = text_editor(&code.content)
        .id(editor_id(pane_id))
        .on_action(move |a| Message::CodeAction(pane_id, a))
        .height(Length::Fill)
        .font(Font::MONOSPACE)
        .size(metrics.font_size)
        .line_height(metrics.cell_h / metrics.font_size)
        .padding([12, 16])
        .style(|_, _status| text_editor::Style {
            background: iced::Background::Color(BG_TERMINAL),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                ..Default::default()
            },
            placeholder: FG_ACTIVE,
            value: FG_ACTIVE,
            selection: iced::Color::from_rgba(0.78, 0.75, 0.84, 0.22),
        });

    let editor = editor.highlight(
        code.language.syntax_token(),
        iced::highlighter::Theme::Base16Ocean,
    );

    let mut children: Vec<Element<Message>> = Vec::new();
    if let Some(hover) = &code.lsp.hover {
        children.push(
            container(
                text(&hover.contents)
                    .font(Font::MONOSPACE)
                    .size(12)
                    .color(FG_ACTIVE)
            )
            .padding([9, 11])
            .style(tooltip_surface_style)
            .into(),
        );
    }
    children.push(editor.into());

    container(column(children).spacing(4))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([8, 10])
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_TERMINAL)),
            ..Default::default()
        })
        .into()
}

use iced::advanced::widget::Id;
use iced::widget::{button, column, container, mouse_area, opaque, row, scrollable, stack, text, text_input, Space};
use iced::{Element, Font, Length};

use crate::file_picker::FilePickerState;
use crate::message::Message;
use crate::style::{BACKDROP_DIM, BG_MENU_HOVER, BG_SURFACE_ALT, BORDER_SOFT, FG_ACTIVE, FG_DIM, FG_INACTIVE};
use crate::widgets::dialog_surface_style;

const PICKER_W: f32 = 1040.0;
const PICKER_H: f32 = 600.0;

pub fn input_id() -> Id {
    Id::new("file-picker-input")
}

pub fn view<'a>(state: &'a FilePickerState) -> Element<'a, Message> {
    let input = text_input("Type to filter files...", &state.query)
        .id(input_id())
        .on_input(Message::FilePickerInput)
        .on_submit(Message::FilePickerOpen)
        .font(Font::MONOSPACE)
        .size(15)
        .padding([12, 14])
        .width(Length::Fill)
        .style(|theme, status| {
            let mut style = text_input::default(theme, status);
            style.background = iced::Background::Color(BG_SURFACE_ALT);
            style.border = iced::Border {
                color: BORDER_SOFT,
                width: 1.0,
                radius: 10.0.into(),
            };
            style.icon = FG_DIM;
            style.placeholder = FG_DIM;
            style.value = FG_ACTIVE;
            style.selection = iced::Color::from_rgba(0.78, 0.75, 0.84, 0.18);
            style
        });

    let mut items: Vec<Element<Message>> = Vec::with_capacity(state.filtered.len());
    for (row_idx, &file_idx) in state.filtered.iter().enumerate() {
        let path = &state.all_files[file_idx];
        let filename = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let parent = path.parent()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        let selected = row_idx == state.selected;
        let item = button(
            column![
                text(filename)
                    .font(Font::MONOSPACE)
                    .size(13)
                    .color(if selected { FG_ACTIVE } else { FG_INACTIVE }),
                text(parent)
                    .font(Font::MONOSPACE)
                    .size(11)
                    .color(FG_DIM),
            ]
            .spacing(2),
        )
            .on_press(Message::FilePickerSelect(row_idx))
            .width(Length::Fill)
            .padding([9, 11])
            .style(move |_, _| button::Style {
                background: if selected { Some(iced::Background::Color(BG_MENU_HOVER)) } else { None },
                text_color: if selected { FG_ACTIVE } else { FG_INACTIVE },
                border: iced::Border { radius: 8.0.into(), ..Default::default() },
                ..Default::default()
            });
        items.push(item.into());
    }
    let list = scrollable(column(items).spacing(4).padding([2, 0]))
        .height(Length::Fill)
        .width(Length::FillPortion(3));

    let preview = container(column![
        text(&state.preview.title)
            .font(Font::MONOSPACE)
            .size(12)
            .color(FG_DIM),
        scrollable(
            text(&state.preview.body)
                .font(Font::MONOSPACE)
                .size(13)
                .color(FG_ACTIVE)
        )
        .height(Length::Fill)
    ]
    .spacing(10))
    .width(Length::FillPortion(2))
    .height(Length::Fill)
    .padding([14, 14])
    .style(|_| container::Style {
        background: Some(iced::Background::Color(BG_SURFACE_ALT)),
        border: iced::Border {
            color: BORDER_SOFT,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    });

    let stats = text(format!("{} files", state.filtered.len()))
        .font(Font::MONOSPACE)
        .size(12)
        .color(FG_DIM);

    let body = row![
        container(list)
            .height(Length::Fill)
            .padding([0, 0]),
        preview,
    ]
    .spacing(16)
    .height(Length::Fill);

    let panel = container(
        column![
            input,
            body,
            stats,
        ]
        .spacing(14)
        .padding([16, 16]),
    )
        .width(Length::Fixed(PICKER_W))
        .height(Length::Fixed(PICKER_H))
        .style(dialog_surface_style);

    let centered = container(panel)
        .width(Length::Fill).height(Length::Fill)
        .center_y(Length::Fill)
        .center_x(Length::Fill);

    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(
                    BACKDROP_DIM,
                )),
                ..Default::default()
            }),
    ).on_press(Message::FilePickerClose);

    stack![row![backdrop].width(Length::Fill).height(Length::Fill), opaque(centered)]
        .width(Length::Fill).height(Length::Fill).into()
}

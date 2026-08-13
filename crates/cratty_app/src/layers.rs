use iced::widget::stack;
use iced::{Element, Length};

use crate::message::Message;

pub fn compose(layers: Vec<Element<'_, Message>>) -> Element<'_, Message> {
    if layers.len() == 1 {
        return layers.into_iter().next().unwrap();
    }
    stack(layers).width(Length::Fill).height(Length::Fill).into()
}

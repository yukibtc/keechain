// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license
use iced::widget::{Column, Container, Row, Scrollable};
use iced::{Alignment, Element, Length};

use super::{rule, Identity};

pub fn view<'a, Message: 'a + Clone + 'static>(
    column: Column<'a, Message>,
    identity: Option<Identity>,
) -> Element<'a, Message> {
    let content = Container::new(
        column
            .align_items(Alignment::Center)
            .spacing(20)
            .padding(20),
    )
    .width(Length::Fill)
    .center_x()
    .center_y()
    .max_width(400);

    let mut v = Column::new();

    if let Some(identity) = identity {
        v = v.push(identity.view()).push(rule::horizontal()).spacing(10);
    }

    v.push(
        Row::new().push(
            Container::new(Scrollable::new(content))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y(),
        ),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use iced::alignment::Horizontal;
use iced::widget::{Container, Row};
use iced::Length;

use crate::app::Message;

pub struct Navbar;

impl Navbar {
    pub fn view<'a>() -> Container<'a, Message> {
        let content = Row::new();
        Container::new(content)
            .width(Length::Fill)
            .padding(20)
            .center_y()
            .align_x(Horizontal::Right)
    }
}

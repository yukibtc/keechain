// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use iced::alignment::Horizontal;
use iced::widget::Text;
use iced::{Font, Length};

const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../../static/icon/bootstrap-icons.otf"),
};

pub struct Icon;

impl Icon {
    pub fn view(unicode: &'static char) -> Text<'static> {
        Text::new(unicode.to_string())
            .font(ICONS)
            .width(Length::Units(20))
            .horizontal_alignment(Horizontal::Center)
            .size(20)
    }
}

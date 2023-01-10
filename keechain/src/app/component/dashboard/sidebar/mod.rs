// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use iced::widget::{Column, Container, Space, Text};
use iced::Length;

mod button;

use self::button::{SidebarButton, BUTTON_SIZE};
use crate::app::{Context, Message, Stage};
use crate::component::Icon;
use crate::theme::icon::{HOME, SETTING};

#[derive(Clone, Default)]
pub struct Sidebar;

impl Sidebar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view<'a>(&self, ctx: &Context) -> Container<'a, Message> {
        let title = Text::new("KeeChain").size(38);
        let home_button =
            SidebarButton::new("Home", Icon::view(&HOME)).view(ctx, Message::View(Stage::Home));
        let settings_button = SidebarButton::new("Settings", Icon::view(&SETTING))
            .view(ctx, Message::View(Stage::Setting));

        let version = Text::new(format!(
            "{} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .size(16);

        sidebar(
            Container::new(
                Column::new()
                    .push(Space::with_height(Length::Units(30)))
                    .push(title)
                    .push(Space::with_height(Length::Units(30)))
                    .padding(15),
            )
            .width(Length::Units(BUTTON_SIZE))
            .center_x(),
            sidebar_menu(vec![home_button, settings_button]),
            sidebar_menu(vec![Container::new(version)
                .width(Length::Units(BUTTON_SIZE))
                .center_x()]),
        )
    }
}

pub fn sidebar<'a, T: 'a>(
    title: Container<'a, T>,
    menu: Container<'a, T>,
    footer: Container<'a, T>,
) -> Container<'a, T> {
    Container::new(
        Column::new()
            .padding(10)
            .push(title)
            .push(menu.height(Length::Fill))
            .push(footer.height(Length::Shrink)),
    )
}

pub fn sidebar_menu<'a, T: 'a>(items: Vec<Container<'a, T>>) -> Container<'a, T> {
    let mut col = Column::new().padding(15).spacing(15);
    for i in items {
        col = col.push(i)
    }
    Container::new(col)
}

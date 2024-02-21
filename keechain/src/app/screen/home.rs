// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use iced::widget::Column;
use iced::{Command, Element, Length};

use crate::app::{Context, Message, Stage, State};
use crate::component::{view, Button, ButtonStyle};
//use crate::theme::icon::{BROADCAST_PIN, KEY, NETWORK, SETTING, TRASH};

#[derive(Debug, Clone)]
pub enum HomeMessage {}

#[derive(Debug, Default)]
pub struct HomeState {}

impl HomeState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl State for HomeState {
    fn title(&self) -> String {
        String::from("Home")
    }

    fn update(&mut self, _ctx: &mut Context, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self, ctx: &Context) -> Element<Message> {
        let content = Column::new()
            .push(
                Button::new()
                    .text("Sign")
                    //.icon(KEY)
                    .style(ButtonStyle::Bordered)
                    .on_press(Message::View(Stage::Sign))
                    .width(Length::Fill)
                    .view(),
            )
            .push(
                Button::new()
                    .text("Passphrase")
                    //.icon(SETTING)
                    .style(ButtonStyle::Bordered)
                    //.on_press(Message::View(Stage::Config))
                    .width(Length::Fill)
                    .view(),
            )
            .push(
                Button::new()
                    .text("Export")
                    //.icon(KEY)
                    .style(ButtonStyle::Bordered)
                    //.on_press(Message::View(Stage::RecoveryKeys))
                    .width(Length::Fill)
                    .view(),
            )
            .push(
                Button::new()
                    .text("Advanced")
                    //.icon(NETWORK)
                    .style(ButtonStyle::Bordered)
                    //.on_press(Message::View(Stage::Relays))
                    .width(Length::Fill)
                    .view(),
            )
            .push(
                Button::new()
                    .text("Settings")
                    //.icon(BROADCAST_PIN)
                    .style(ButtonStyle::Bordered)
                    //.on_press(HomeMessage::RebroadcastAllEvents.into())
                    .width(Length::Fill)
                    .view(),
            )
            .push(
                Button::new()
                    .text("Lock")
                    //.icon(TRASH)
                    .style(ButtonStyle::BorderedDanger)
                    .on_press(Message::Lock)
                    .width(Length::Fill)
                    .view(),
            )
            .spacing(10);

        view(content, Some(ctx.identity()))
    }
}

impl From<HomeState> for Box<dyn State> {
    fn from(s: HomeState) -> Box<dyn State> {
        Box::new(s)
    }
}

impl From<HomeMessage> for Message {
    fn from(msg: HomeMessage) -> Self {
        Self::Home(msg)
    }
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use iced::widget::{button, column, container, pick_list, row, text_input, Text};
use iced::{Command, Element, Length};
use keechain_core::keychain::KeeChain;
use keechain_core::util::dir;

use crate::start::{Context, Message, State};
use crate::theme::color::DARK_RED;

#[derive(Debug, Clone)]
pub enum OpenMessage {
    LoadKeychains,
    KeychainSelect(String),
    PasswordChanged(String),
    OpenButtonPressed,
}

#[derive(Debug, Default)]
pub struct OpenState {
    keychains: Vec<String>,
    name: Option<String>,
    password: String,
    error: Option<String>,
}

impl OpenState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl State for OpenState {
    fn title(&self) -> String {
        String::from("KeeChain - Open")
    }

    fn load(&mut self, _ctx: &Context) -> Command<Message> {
        Command::perform(async {}, |_| Message::Open(OpenMessage::LoadKeychains))
    }

    fn update(&mut self, _ctx: &mut Context, message: Message) -> Command<Message> {
        if let Message::Open(msg) = message {
            match msg {
                OpenMessage::LoadKeychains => match dir::get_keychains_list() {
                    Ok(list) => self.keychains = list,
                    Err(e) => self.error = Some(e.to_string()),
                },
                OpenMessage::KeychainSelect(name) => self.name = Some(name),
                OpenMessage::PasswordChanged(psw) => self.password = psw,
                OpenMessage::OpenButtonPressed => {
                    if let Some(name) = &self.name {
                        match KeeChain::open(name, || Ok(self.password.clone())) {
                            Ok(keechain) => {
                                return Command::perform(async {}, move |_| {
                                    Message::OpenResult(keechain)
                                })
                            }
                            Err(e) => self.error = Some(e.to_string()),
                        }
                    } else {
                        self.error = Some(String::from("Please, select a keychain"));
                    }
                }
            }
        };

        Command::none()
    }

    fn view(&self, _ctx: &Context) -> Element<Message> {
        let keychain_pick_list = pick_list(self.keychains.clone(), self.name.clone(), |name| {
            Message::Open(OpenMessage::KeychainSelect(name))
        })
        .placeholder(if self.keychains.is_empty() {
            "No keychain availabe"
        } else {
            "Select a keychain"
        });

        let password = text_input("Password", &self.password, |s| {
            Message::Open(OpenMessage::PasswordChanged(s))
        })
        .on_submit(Message::Open(OpenMessage::OpenButtonPressed))
        .padding(10)
        .password()
        .size(20);

        let button = button("Open")
            .padding(10)
            .on_press(Message::Open(OpenMessage::OpenButtonPressed));

        let content = column![
            row![keychain_pick_list],
            row![password, button].spacing(10),
            if let Some(error) = &self.error {
                row![Text::new(error).style(DARK_RED)]
            } else {
                row![]
            }
        ]
        .spacing(20)
        .padding(20)
        .max_width(600);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

impl From<OpenState> for Box<dyn State> {
    fn from(s: OpenState) -> Box<dyn State> {
        Box::new(s)
    }
}

// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use iced::widget::{Column, Row};
use iced::{Command, Element, Length};
use keechain_core::{KeeChain, WordCount};

use crate::component::{rule, view, Button, ButtonStyle, Text, TextInput};
use crate::start::{Context, Message, Stage, State};
use crate::theme::color::DARK_RED;
use crate::{BASE_PATH, SECP256K1};

#[derive(Debug, Clone)]
pub enum GenerateMessage {
    NameChanged(String),
    PasswordChanged(String),
    ConfirmPasswordChanged(String),
    ErrorChanged(Option<String>),
    Generate,
}

#[derive(Debug, Default)]
pub struct GenerateState {
    name: String,
    password: String,
    confirm_password: String,
    // mnemonic: Option<Mnemonic>,
    error: Option<String>,
}

impl GenerateState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl State for GenerateState {
    fn title(&self) -> String {
        String::from("Generate")
    }

    fn update(&mut self, ctx: &mut Context, message: Message) -> Command<Message> {
        if let Message::Generate(msg) = message {
            match msg {
                GenerateMessage::NameChanged(name) => self.name = name,
                GenerateMessage::PasswordChanged(passwd) => self.password = passwd,
                GenerateMessage::ConfirmPasswordChanged(passwd) => self.confirm_password = passwd,
                GenerateMessage::ErrorChanged(e) => {
                    self.error = e;
                }
                GenerateMessage::Generate => {
                    let network = ctx.network;
                    let name = self.name.clone();
                    let password = self.password.clone();
                    let confirm_password = self.confirm_password.clone();
                    return Command::perform(
                        async move {
                            KeeChain::generate(
                                BASE_PATH.as_path(),
                                name,
                                || Ok(password),
                                || Ok(confirm_password),
                                WordCount::W12, // TODO: let user choose the len.
                                || Ok(None),
                                network,
                                &SECP256K1,
                            )
                        },
                        move |res| match res {
                            Ok(keechain) => Message::OpenResult(Box::new(keechain)),
                            Err(e) => GenerateMessage::ErrorChanged(Some(e.to_string())).into(),
                        },
                    );
                }
            }
        };

        Command::none()
    }

    fn view(&self, _ctx: &Context) -> Element<Message> {
        let name = TextInput::new(&self.name)
            .label("Name")
            .on_input(|s| GenerateMessage::NameChanged(s).into())
            .placeholder("Name of keychain")
            .view();

        let password = TextInput::new(&self.password)
            .label("Password")
            .on_input(|s| GenerateMessage::PasswordChanged(s).into())
            .placeholder("Password")
            .password()
            .view();

        let confirm_password = TextInput::new(&self.confirm_password)
            .label("Confirm password")
            .on_input(|s| GenerateMessage::ConfirmPasswordChanged(s).into())
            .placeholder("Confirm password")
            .password()
            .view();

        let generate_keychain_btn = Button::new()
            .text("Generate")
            .on_press(GenerateMessage::Generate.into())
            .width(Length::Fill);

        let open_btn = Button::new()
            .text("Open keychain")
            .style(ButtonStyle::Bordered)
            .width(Length::Fill)
            .on_press(Message::View(Stage::Open));

        let restore_keychain_btn = Button::new()
            .text("Restore keychain")
            .style(ButtonStyle::Bordered)
            .on_press(Message::View(Stage::Restore))
            .width(Length::Fill);

        let content = Column::new()
            .push(name)
            .push(password)
            .push(confirm_password)
            .push(if let Some(error) = &self.error {
                Row::new().push(Text::new(error).color(DARK_RED).view())
            } else {
                Row::new()
            })
            .push(generate_keychain_btn.view())
            .push(rule::horizontal())
            .push(open_btn.view())
            .push(restore_keychain_btn.view());

        view(content, None)
    }
}

impl From<GenerateState> for Box<dyn State> {
    fn from(s: GenerateState) -> Box<dyn State> {
        Box::new(s)
    }
}

impl From<GenerateMessage> for Message {
    fn from(msg: GenerateMessage) -> Self {
        Self::Generate(msg)
    }
}

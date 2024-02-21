// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use iced::widget::{Column, Row};
use iced::{Command, Element, Length};
use keechain_core::bips::bip39::Mnemonic;
use keechain_core::KeeChain;

use crate::component::{rule, view, Button, ButtonStyle, Text, TextInput};
use crate::start::{Context, Message, Stage, State};
use crate::theme::color::DARK_RED;
use crate::{BASE_PATH, SECP256K1};

#[derive(Debug, Clone)]
pub enum RestoreMessage {
    NameChanged(String),
    PasswordChanged(String),
    ConfirmPasswordChanged(String),
    MnemonicChanged(String),
    ErrorChanged(Option<String>),
    RestoreButtonPressed,
}

#[derive(Debug, Default)]
pub struct RestoreState {
    name: String,
    password: String,
    confirm_password: String,
    mnemonic: String,
    error: Option<String>,
}

impl RestoreState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl State for RestoreState {
    fn title(&self) -> String {
        String::from("Restore")
    }

    fn update(&mut self, ctx: &mut Context, message: Message) -> Command<Message> {
        if let Message::Restore(msg) = message {
            match msg {
                RestoreMessage::NameChanged(name) => self.name = name,
                RestoreMessage::PasswordChanged(passwd) => self.password = passwd,
                RestoreMessage::ConfirmPasswordChanged(passwd) => self.confirm_password = passwd,
                RestoreMessage::MnemonicChanged(mnemonic) => self.mnemonic = mnemonic,
                RestoreMessage::ErrorChanged(e) => {
                    self.error = e;
                }
                RestoreMessage::RestoreButtonPressed => {
                    let network = ctx.network;
                    let name = self.name.clone();
                    let password = self.password.clone();
                    let confirm_password = self.confirm_password.clone();
                    let mnemonic = self.mnemonic.clone();
                    return Command::perform(
                        async move {
                            KeeChain::restore(
                                BASE_PATH.as_path(),
                                name,
                                || Ok(password),
                                || Ok(confirm_password),
                                || Ok(Mnemonic::from_str(&mnemonic)?),
                                network,
                                &SECP256K1,
                            )
                        },
                        move |res| match res {
                            Ok(keechain) => Message::OpenResult(Box::new(keechain)),
                            Err(e) => RestoreMessage::ErrorChanged(Some(e.to_string())).into(),
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
            .on_input(|s| Message::Restore(RestoreMessage::NameChanged(s)))
            .placeholder("Name of keychain")
            .view();

        let password = TextInput::new(&self.password)
            .label("Password")
            .on_input(|s| Message::Restore(RestoreMessage::PasswordChanged(s)))
            .placeholder("Password")
            .password()
            .view();

        let confirm_password = TextInput::new(&self.confirm_password)
            .label("Confirm password")
            .on_input(|s| Message::Restore(RestoreMessage::ConfirmPasswordChanged(s)))
            .placeholder("Confirm password")
            .password()
            .view();

        let mnemonic = TextInput::new(&self.mnemonic)
            .label("Mnemonic (BIP39)")
            .on_input(|s| Message::Restore(RestoreMessage::MnemonicChanged(s)))
            .placeholder("Mnemonic")
            .view();

        let restore_keychain_btn = Button::new()
            .text("Restore")
            .on_press(Message::Restore(RestoreMessage::RestoreButtonPressed))
            .width(Length::Fill)
            .view();

        let open_btn = Button::new()
            .text("Open keychain")
            .style(ButtonStyle::Bordered)
            .width(Length::Fill)
            .on_press(Message::View(Stage::Open))
            .view();

        let new_keychain_btn = Button::new()
            .text("Create keychain")
            .style(ButtonStyle::Bordered)
            .on_press(Message::View(Stage::New))
            .width(Length::Fill)
            .view();

        let content = Column::new()
            .push(name)
            .push(password)
            .push(confirm_password)
            .push(mnemonic)
            .push(if let Some(error) = &self.error {
                Row::new().push(Text::new(error).color(DARK_RED).view())
            } else {
                Row::new()
            })
            .push(restore_keychain_btn)
            .push(rule::horizontal())
            .push(open_btn)
            .push(new_keychain_btn);

        view(content, None)
    }
}

impl From<RestoreState> for Box<dyn State> {
    fn from(s: RestoreState) -> Box<dyn State> {
        Box::new(s)
    }
}

impl From<RestoreMessage> for Message {
    fn from(msg: RestoreMessage) -> Self {
        Self::Restore(msg)
    }
}

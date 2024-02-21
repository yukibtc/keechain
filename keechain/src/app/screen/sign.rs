// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::path::PathBuf;

use iced::widget::{Column, Space};
use iced::{Command, Element, Length};
use keechain_core::bitcoin::psbt::PartiallySignedTransaction;
use keechain_core::types::keechain;
use keechain_core::util::dir;
use keechain_core::{psbt, KeeChain, PsbtUtility};
use rfd::FileDialog;
use thiserror::Error;

use crate::app::{Context, Message, Stage, State};
use crate::component::{view, Button, ButtonStyle, Text, TextInput};
use crate::theme::color::{GREEN, RED};
use crate::SECP256K1;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Dir(#[from] dir::Error),
    #[error(transparent)]
    Psbt(#[from] psbt::Error),
    #[error(transparent)]
    Keechain(#[from] keechain::Error),
    #[error("No PSBT selected")]
    NoPsbtSelected,
}

fn sign_psbt(
    keechain: KeeChain,
    password: String,
    psbt: PsbtWithPath,
) -> Result<PartiallySignedTransaction, Error> {
    let PsbtWithPath { mut psbt, mut path } = psbt;
    let finalized: bool = keechain.sign_psbt(password, &mut psbt, None, Vec::new(), &SECP256K1)?;
    dir::rename_psbt(&mut path, finalized)?;
    psbt.save_to_file(path)?;
    Ok(psbt)
}

#[derive(Debug, Clone)]
pub enum SignMessage {
    SelectPsbt,
    PsbtSelected(PsbtWithPath),
    PasswordChanged(String),
    CustomDescriptorChanged(String),
    Sign,
    Clear,
    Signed(PartiallySignedTransaction),
    //ExportSignedPsbtToFile,
    ErrorChanged(Option<String>),
}

#[derive(Debug, Clone)]
pub struct PsbtWithPath {
    psbt: PartiallySignedTransaction,
    path: PathBuf,
}

impl Deref for PsbtWithPath {
    type Target = PartiallySignedTransaction;

    fn deref(&self) -> &Self::Target {
        &self.psbt
    }
}

#[derive(Debug, Default)]
pub struct SignState {
    psbt: Option<PsbtWithPath>,
    password: String,
    custom_descriptor: String,
    error: Option<String>,
    signed_psbt: Option<PartiallySignedTransaction>,
}

impl SignState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl State for SignState {
    fn title(&self) -> String {
        String::from("Sign")
    }

    fn update(&mut self, ctx: &mut Context, message: Message) -> Command<Message> {
        if let Message::Sign(msg) = message {
            match msg {
                SignMessage::SelectPsbt => {
                    return Command::perform(
                        async move {
                            let path = FileDialog::new()
                                .add_filter("psbt", &["psbt"])
                                .pick_file()
                                .ok_or(Error::NoPsbtSelected)?;
                            Ok::<PsbtWithPath, Error>(PsbtWithPath {
                                psbt: PartiallySignedTransaction::from_file(&path)?,
                                path,
                            })
                        },
                        |res| match res {
                            Ok(psbt) => SignMessage::PsbtSelected(psbt).into(),
                            Err(e) => SignMessage::ErrorChanged(Some(e.to_string())).into(),
                        },
                    )
                }
                SignMessage::PsbtSelected(psbt) => self.psbt = Some(psbt),
                SignMessage::PasswordChanged(password) => self.password = password,
                SignMessage::CustomDescriptorChanged(descriptor) => {
                    self.custom_descriptor = descriptor
                }
                SignMessage::Sign => {
                    let keechain = ctx.keechain.clone();
                    let password = self.password.clone();
                    match self.psbt.clone() {
                        Some(psbt) => {
                            return Command::perform(
                                async move { sign_psbt(keechain, password, psbt) },
                                |res| match res {
                                    Ok(psbt) => SignMessage::Signed(psbt).into(),
                                    Err(e) => SignMessage::ErrorChanged(Some(e.to_string())).into(),
                                },
                            )
                        }
                        None => {
                            self.error = Some(String::from("PSBT not selected!"));
                        }
                    }
                }
                SignMessage::Clear => {
                    self.psbt = None;
                    self.error = None;
                    self.signed_psbt = None;
                }
                SignMessage::Signed(psbt) => {
                    self.error = None;
                    self.signed_psbt = Some(psbt);
                }
                /* SignMessage::ExportSignedPsbtToFile => {
                    if let Some(signed_psbt) = self.signed_psbt.clone() {
                        return Command::perform(
                            async move {
                                if let Some(path) =
                                    FileDialog::new().add_filter("psbt", &["psbt"]).save_file()
                                {
                                    signed_psbt.save_to_file(path)?;
                                }
                                Ok::<(), Error>(())
                            },
                            |res| match res {
                                Ok(..) => Message::Tick,
                                Err(e) => SignMessage::ErrorChanged(Some(e.to_string())).into(),
                            },
                        );
                    }
                } */
                SignMessage::ErrorChanged(e) => self.error = e,
            }
        }

        Command::none()
    }

    fn view(&self, ctx: &Context) -> Element<Message> {
        let mut content = Column::new().push(match &self.error {
            Some(e) => Text::new(e).color(RED).view(),
            None => Text::new("").view(),
        });

        match &self.psbt {
            Some(..) => {
                match &self.signed_psbt {
                    Some(..) => {
                        content = content
                            .push(Text::new("Signed!").size(24).bold().color(GREEN).view())
                            .push(Space::with_height(Length::Fixed(10.0)))
                        /* .push(
                            Button::new()
                                .text("Export signed PSBT")
                                .on_press(SignMessage::ExportSignedPsbtToFile.into())
                                .width(Length::Fill)
                                .view(),
                        ) */
                    }
                    None => {
                        content = content
                            .push(
                                TextInput::new(&self.password)
                                    .label("Password")
                                    .placeholder("Password")
                                    .password()
                                    .on_input(|p| SignMessage::PasswordChanged(p).into())
                                    .view(),
                            )
                            .push(
                                Button::new()
                                    .text("Sign")
                                    .on_press(SignMessage::Sign.into())
                                    .width(Length::Fill)
                                    .view(),
                            )
                    }
                }

                content = content.push(
                    Button::new()
                        .text("Clear")
                        .style(ButtonStyle::BorderedDanger)
                        .on_press(SignMessage::Clear.into())
                        .width(Length::Fill)
                        .view(),
                );
            }
            None => {
                content = content.push(
                    Button::new()
                        .text("Select PSBT")
                        .on_press(SignMessage::SelectPsbt.into())
                        .width(Length::Fill)
                        .view(),
                );
            }
        }

        content = content
            .push(
                Button::new()
                    .text("Back")
                    .style(ButtonStyle::Bordered)
                    .on_press(Message::View(Stage::Home))
                    .width(Length::Fill)
                    .view(),
            )
            .spacing(10);

        view(content, Some(ctx.identity()))
    }
}

impl From<SignState> for Box<dyn State> {
    fn from(s: SignState) -> Box<dyn State> {
        Box::new(s)
    }
}

impl From<SignMessage> for Message {
    fn from(msg: SignMessage) -> Self {
        Self::Sign(msg)
    }
}

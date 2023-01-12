// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;

use iced::widget::qr_code::{self, QRCode};
use iced::widget::{Column, Row};
use iced::{Command, Element, Length};
use keechain_core::types::Psbt;
use keechain_core::util::dir;
use rfd::FileDialog;

use crate::app::component::Dashboard;
use crate::app::{Context, Message, State};
use crate::component::{button, Icon, Text};
use crate::theme::color::{DARK_GREEN, DARK_RED};
use crate::theme::icon::CHECK_CIRCLE;

#[derive(Debug, Clone)]
pub enum SignMessage {
    SelectPsbtFile,
    Sign,
    Clear,
    ExportWithQrCode,
    SaveToFile,
}

#[derive(Debug, Clone)]
pub struct PsbtFile {
    psbt: Psbt,
    path: PathBuf,
}

#[derive(Debug, Default)]
pub struct SignState {
    psbt_file: Option<PsbtFile>,
    finish: bool,
    qr_code: Option<qr_code::State>,
    error: Option<String>,
}

impl SignState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.psbt_file = None;
        self.finish = false;
        self.qr_code = None;
        self.error = None;
    }
}

impl State for SignState {
    fn title(&self) -> String {
        String::from("KeeChain - Sign")
    }

    fn update(&mut self, ctx: &mut Context, message: Message) -> Command<Message> {
        if let Message::Sign(msg) = message {
            match msg {
                SignMessage::SelectPsbtFile => {
                    if let Some(path) = FileDialog::new().add_filter("psbt", &["psbt"]).pick_file()
                    {
                        match Psbt::from_file(path.clone(), ctx.network) {
                            Ok(psbt) => {
                                self.error = None;
                                self.psbt_file = Some(PsbtFile { psbt, path });
                            }
                            Err(e) => self.error = Some(e.to_string()),
                        }
                    }
                }
                SignMessage::Sign => {
                    if let Some(psbt_file) = self.psbt_file.as_mut() {
                        match psbt_file.psbt.sign(&ctx.keechain.keychain.seed) {
                            Ok(finalized) => {
                                if finalized {
                                    self.error = None;
                                    self.finish = true;
                                } else {
                                    self.error = Some("PSBT signing not finalized".to_string());
                                }
                            }
                            Err(e) => self.error = Some(e.to_string()),
                        }
                    } else {
                        self.error = Some("PSBT not loaded".to_string());
                    }
                }
                SignMessage::Clear => self.clear(),
                SignMessage::SaveToFile => {
                    if let Some(psbt_file) = self.psbt_file.as_mut() {
                        let mut path: PathBuf = psbt_file.path.clone();
                        match dir::rename_psbt_to_signed(&mut path) {
                            Ok(_) => match psbt_file.psbt.save_to_file(path) {
                                Ok(_) => self.clear(),
                                Err(e) => self.error = Some(e.to_string()),
                            },
                            Err(e) => self.error = Some(e.to_string()),
                        }
                    } else {
                        self.error = Some("PSBT not loaded".to_string());
                    }
                }
                SignMessage::ExportWithQrCode => {
                    if let Some(psbt_file) = self.psbt_file.as_ref() {
                        self.qr_code = qr_code::State::new(psbt_file.psbt.as_base64()).ok();
                    } else {
                        self.error = Some("PSBT not loaded".to_string());
                    }
                }
            }
        }

        Command::none()
    }

    fn view(&self, ctx: &Context) -> Element<Message> {
        let mut content = Column::new();

        if self.finish {
            if let Some(qr_code) = self.qr_code.as_ref() {
                let clearn_button =
                    button::primary("Clear").on_press(Message::Sign(SignMessage::Clear));

                content = content
                    .push(QRCode::new(qr_code).cell_size(2))
                    .push(clearn_button)
                    .spacing(20);
            } else {
                let icon = Icon::new(&CHECK_CIRCLE)
                    .width(Length::Units(100))
                    .size(100)
                    .style(DARK_GREEN);

                let save_to_file = button::primary("Save to file")
                    .on_press(Message::Sign(SignMessage::SaveToFile));

                let show_qr_code = button::primary("Show QR Code")
                    .on_press(Message::Sign(SignMessage::ExportWithQrCode));

                content = content
                    .push(icon)
                    .push(Row::new().push(save_to_file).push(show_qr_code).spacing(10))
                    .spacing(20);
            }
        } else if self.psbt_file.is_some() {
            let sign_button = button::primary("Sign").on_press(Message::Sign(SignMessage::Sign));
            let clear_button = button::border("Clear").on_press(Message::Sign(SignMessage::Clear));

            content = content.push(sign_button).push(clear_button).spacing(10);
        } else {
            let select_button = button::primary("Select PSBT file")
                .on_press(Message::Sign(SignMessage::SelectPsbtFile));

            content = content.push(select_button);
        }

        if let Some(error) = &self.error {
            content = content.push(Text::new(error).color(DARK_RED).view());
        }

        Dashboard::new().view(ctx, content)
    }
}

impl From<SignState> for Box<dyn State> {
    fn from(s: SignState) -> Box<dyn State> {
        Box::new(s)
    }
}

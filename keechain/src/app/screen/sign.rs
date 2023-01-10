// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;

use iced::widget::Column;
use iced::{Command, Element, Length};
use keechain_core::command;
use keechain_core::types::Psbt;
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
    error: Option<String>,
}

impl SignState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.psbt_file = None;
        self.finish = false;
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
                        match command::psbt::decode_file(path.clone(), ctx.network) {
                            Ok(psbt) => {
                                self.error = None;
                                self.psbt_file = Some(PsbtFile { psbt, path });
                            }
                            Err(e) => self.error = Some(e.to_string()),
                        }
                    }
                }
                SignMessage::Sign => match command::psbt::sign_file_from_seed(
                    &ctx.keechain.keychain.seed(),
                    ctx.network,
                    self.psbt_file.clone().unwrap().path,
                ) {
                    Ok(finalized) => {
                        if finalized {
                            self.clear();
                            self.finish = true;
                        } else {
                            self.error = Some("PSBT signing not finalized".to_string());
                        }
                    }
                    Err(e) => self.error = Some(e.to_string()),
                },
                SignMessage::Clear => self.clear(),
            }
        }

        Command::none()
    }

    fn view(&self, ctx: &Context) -> Element<Message> {
        let mut content = Column::new();

        if self.finish {
            let icon = Icon::new(&CHECK_CIRCLE)
                .width(Length::Units(100))
                .size(100)
                .style(DARK_GREEN);
            let sign_again_button =
                button::primary("Sign again").on_press(Message::Sign(SignMessage::Clear));

            content = content.push(icon).push(sign_again_button).spacing(20);
        } else if self.psbt_file.is_some() {
            let sign_button = button::primary("Sign").on_press(Message::Sign(SignMessage::Sign));
            let clear_button = button::border("Clear").on_press(Message::Sign(SignMessage::Clear));

            content = content.push(sign_button).push(clear_button).spacing(10);

            if let Some(error) = &self.error {
                content = content.push(Text::new(error).color(DARK_RED).view());
            }
        } else {
            let select_button = button::primary("Select PSBT file")
                .on_press(Message::Sign(SignMessage::SelectPsbtFile));

            content = content.push(select_button);
        }

        Dashboard::new().view(ctx, content)
    }
}

impl From<SignState> for Box<dyn State> {
    fn from(s: SignState) -> Box<dyn State> {
        Box::new(s)
    }
}

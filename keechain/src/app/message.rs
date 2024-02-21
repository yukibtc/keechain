// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use super::screen::*;
use super::Stage;

#[derive(Debug, Clone)]
pub enum Message {
    View(Stage),
    Home(HomeMessage),
    Sign(SignMessage),
    Clipboard(String),
    Lock,
    Tick,
}

impl From<Message> for crate::Message {
    fn from(msg: Message) -> Self {
        Self::App(Box::new(msg))
    }
}

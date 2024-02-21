// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use keechain_core::KeeChain;

use super::screen::{GenerateMessage, OpenMessage, RestoreMessage};
use super::Stage;

#[derive(Debug, Clone)]
pub enum Message {
    View(Stage),
    Open(OpenMessage),
    Restore(RestoreMessage),
    Generate(GenerateMessage),
    OpenResult(Box<KeeChain>),
    Load,
}

impl From<Message> for crate::Message {
    fn from(msg: Message) -> Self {
        Self::Start(Box::new(msg))
    }
}

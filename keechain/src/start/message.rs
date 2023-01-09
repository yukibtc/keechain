// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use keechain_core::keychain::KeeChain;

use super::screen::OpenMessage;
use super::Stage;

#[derive(Debug, Clone)]
pub enum Message {
    View(Stage),
    Open(OpenMessage),
    OpenResult(KeeChain),
}

impl From<Message> for crate::Message {
    fn from(msg: Message) -> Self {
        Self::Start(Box::new(msg))
    }
}

// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use iced::{widget::Column, Alignment, Length};
use keechain_core::bips::bip32::Fingerprint;

use super::Text;

pub struct Identity {
    fingerprint: Fingerprint,
    passphrase: bool,
}

impl Identity {
    pub fn new(fingerprint: Fingerprint, passphrase: bool) -> Self {
        Self {
            fingerprint,
            passphrase,
        }
    }

    pub fn view<'a, Message: Clone + 'static>(self) -> Column<'a, Message> {
        Column::new()
            .push(
                Text::new(format!("Fingerprint: {}", self.fingerprint))
                    .width(Length::Fill)
                    .view(),
            )
            .push(
                Text::new(format!("Passphrase: {}", self.passphrase))
                    .width(Length::Fill)
                    .view(),
            )
            .align_items(Alignment::Center)
    }
}

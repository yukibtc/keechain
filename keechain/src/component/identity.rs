// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{RichText, Ui};
use keechain_core::bitcoin::bip32::Fingerprint;

pub struct Identity {
    fingerprint: Fingerprint,
    passphrase: bool,
}

impl Identity {
    pub fn new(fingerprint: Fingerprint, passphrase: Option<String>) -> Self {
        Self {
            fingerprint,
            passphrase: passphrase.is_some(),
        }
    }

    pub fn render(self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new(format!("Fingerprint: {}", self.fingerprint)).small());
            ui.label(RichText::new(format!("Using a passphrase: {}", self.passphrase)).small());
        });
    }
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{RichText, Ui};
use keechain_core::bitcoin::Network;
use keechain_core::types::Seed;
use keechain_core::util::bip::bip32::Bip32RootKey;

pub struct Identity {
    seed: Seed,
    network: Network,
}

impl Identity {
    pub fn new(seed: Seed, network: Network) -> Self {
        Self { seed, network }
    }

    pub fn render(self, ui: &mut Ui) {
        ui.group(|ui| {
            if let Ok(fingerprint) = self.seed.fingerprint(self.network) {
                ui.label(RichText::new(format!("Fingerprint: {fingerprint}")).small());
            }
            ui.label(
                RichText::new(format!(
                    "Using a passphrase: {}",
                    self.seed.passphrase().is_some()
                ))
                .small(),
            );
        });
    }
}

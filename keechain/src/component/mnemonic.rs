// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::style::Spacing;
use eframe::egui::{Grid, Ui};
use keechain_core::bdk::keys::bip39::Mnemonic;

pub struct MnemonicViewer {
    mnemonic: Mnemonic,
}

impl MnemonicViewer {
    pub fn new(mnemonic: Mnemonic) -> Self {
        Self { mnemonic }
    }

    pub fn render(self, ui: &mut Ui) {
        ui.group(|ui| {
            let colunm_size: usize = self.mnemonic.word_count() / 2;
            let words: Vec<&str> = self.mnemonic.word_iter().collect();
            Grid::new("mnemonic_viewer")
                .min_col_width((ui.available_width() - Spacing::default().item_spacing.x) / 2.0)
                .show(ui, |ui| {
                    for index in 0..colunm_size {
                        if let Some(word) = words.get(index) {
                            ui.label(format!("{}. {}", index + 1, word));
                        }
                        let index = index + colunm_size;
                        if let Some(word) = words.get(index) {
                            ui.label(format!("{}. {}", index + 1, word));
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

mod bitcoin_core;
mod descriptors;
mod electrum;

use crate::{AppState, ExportTypes};

pub fn update_layout(app: &mut AppState, export_type: ExportTypes, ui: &mut Ui) {
    match export_type {
        ExportTypes::Descriptors => self::descriptors::update_layout(app, ui),
        ExportTypes::BitcoinCore => self::bitcoin_core::update_layout(app, ui),
        ExportTypes::Electrum => self::electrum::update_layout(app, ui),
    }
}

// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

pub mod bitcoin_core;
pub mod descriptors;
pub mod electrum;

use crate::{AppState, ExportTypes};

pub fn update(app: &mut AppState, export_type: ExportTypes, ui: &mut Ui) {
    match export_type {
        ExportTypes::Descriptors => self::descriptors::update(app, ui),
        ExportTypes::BitcoinCore => self::bitcoin_core::update(app, ui),
        ExportTypes::Electrum => self::electrum::update(app, ui),
    }
}

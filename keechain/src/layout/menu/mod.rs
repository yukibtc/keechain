// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;
use eframe::Frame;

mod advanced;
mod export;
mod main;
#[cfg(feature = "nostr")]
mod nostr;
mod setting;

use crate::{AppState, Menu};

pub fn update_layout(app: &mut AppState, menu: Menu, ui: &mut Ui, frame: &mut Frame) {
    match menu {
        Menu::Main => self::main::update_layout(app, ui, frame),
        Menu::Export => self::export::update_layout(app, ui),
        Menu::Advanced => self::advanced::update_layout(app, ui),
        Menu::Setting => self::setting::update_layout(app, ui),
        Menu::Danger => self::advanced::danger::update_layout(app, ui),
        #[cfg(feature = "nostr")]
        Menu::Nostr => self::nostr::update_layout(app, ui),
    }
}

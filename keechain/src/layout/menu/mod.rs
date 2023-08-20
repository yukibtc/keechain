// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;
use eframe::Frame;

mod advanced;
mod export;
mod main;
mod setting;

use crate::{AppState, Menu};

pub fn update(app: &mut AppState, menu: Menu, ui: &mut Ui, frame: &mut Frame) {
    match menu {
        Menu::Main => self::main::update(app, ui, frame),
        Menu::Export => self::export::update(app, ui),
        Menu::Advanced => self::advanced::update(app, ui),
        Menu::Setting => self::setting::update(app, ui),
        Menu::Danger => self::advanced::danger::update(app, ui),
    }
}

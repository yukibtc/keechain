// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

use crate::component::{Button, Heading, Identity, View};
use crate::{AppState, Command, ExportTypes, Menu, Stage};

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Export").render(ui);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
        }

        if Button::new("Descriptors").render(ui).clicked() {
            app.set_stage(Stage::Command(Command::Export(ExportTypes::Descriptors)));
        }
        ui.add_space(5.0);
        if Button::new("Bitcoin Core").render(ui).clicked() {
            app.set_stage(Stage::Command(Command::Export(ExportTypes::BitcoinCore)));
        }
        ui.add_space(5.0);
        if Button::new("Electrum").render(ui).clicked() {
            app.set_stage(Stage::Command(Command::Export(ExportTypes::Electrum)));
        }
        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Main);
        }
    });
}

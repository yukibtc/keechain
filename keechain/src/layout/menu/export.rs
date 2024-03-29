// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

use crate::component::{Button, Heading, Identity, View};
use crate::{AppState, Command, ExportTypes, Menu, Stage};

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Export").render(ui);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.identity(), keechain.passphrase()).render(ui);
            ui.add_space(15.0);
        }

        if Button::new("Descriptors")
            .enabled(false)
            .render(ui)
            .clicked()
        {
            app.set_stage(Stage::Command(Command::Export(ExportTypes::Descriptors)));
        }
        ui.add_space(5.0);
        if Button::new("Bitcoin Core")
            .enabled(false)
            .render(ui)
            .clicked()
        {
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

// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

use crate::component::{Button, Heading, Identity, View};
use crate::{AppState, Menu, Stage};

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Export Bitcoin Core descriptors").render(ui);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.identity(), keechain.passphrase()).render(ui);
            ui.add_space(15.0);
        }

        if Button::new("Back").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Export);
        }
    });
}

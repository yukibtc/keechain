// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

use crate::component::{Button, Heading, Identity, View};
use crate::{AppState, Command, Menu, Stage};

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Setting").render(ui);

        ui.add_space(15.0);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
        }

        if Button::new("Rename keychain").render(ui).clicked() {
            app.stage = Stage::Command(Command::RenameKeychain);
        }
        ui.add_space(5.0);
        if Button::new("Change password").render(ui).clicked() {
            app.stage = Stage::Command(Command::ChangePassword);
        }
        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Main);
        }
    });
}

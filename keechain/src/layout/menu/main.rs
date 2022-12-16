// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;
use eframe::Frame;

use crate::component::{Button, Heading, Identity, View};
use crate::{AppState, Command, Menu, Stage};

pub fn update_layout(app: &mut AppState, ui: &mut Ui, frame: &mut Frame) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Menu").render(ui);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
        }

        if Button::new("Sign").render(ui).clicked() {
            app.set_stage(Stage::Command(Command::Sign));
        }
        ui.add_space(5.0);
        if Button::new("Passphrase").render(ui).clicked() {
            app.set_stage(Stage::Command(Command::Passphrase));
        }
        ui.add_space(5.0);
        if Button::new("Export").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Export);
        }
        ui.add_space(5.0);
        if Button::new("Advanced").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Advanced);
        }
        ui.add_space(5.0);
        if Button::new("Setting").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Setting);
        }
        ui.add_space(5.0);
        if Button::new("Lock").render(ui).clicked() {
            app.stage = Stage::Start;
        }
        ui.add_space(5.0);
        if Button::new("Exit").render(ui).clicked() {
            frame.close();
        }
    });
}

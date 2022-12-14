// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

use crate::component::{Button, Heading, Identity, View};
use crate::theme::color::DARK_RED;
use crate::{AppState, Command, Menu, Stage};

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Danger").render(ui);

        ui.add_space(15.0);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
        }

        if Button::new("View secrets")
            .background_color(DARK_RED)
            .render(ui)
            .clicked()
        {
            app.stage = Stage::Command(Command::ViewSecrets);
        }
        ui.add_space(5.0);
        if Button::new("Delete keychain")
            .background_color(DARK_RED)
            .render(ui)
            .clicked()
        {
            app.stage = Stage::Command(Command::WipeKeychain);
        }
        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Advanced);
        }
    });
}

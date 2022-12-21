// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

use crate::component::{Button, Heading, Identity, View};
use crate::{AppState, Command, Menu, Stage};

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Nostr").render(ui);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
        }

        if Button::new("Keys").render(ui).clicked() {
            app.set_stage(Stage::Command(Command::NostrKeys));
        }
        ui.add_space(5.0);
        if Button::new("Sign delegation").render(ui).clicked() {
            app.set_stage(Stage::Command(Command::NostrSignDelegation));
        }
        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Advanced);
        }
    });
}

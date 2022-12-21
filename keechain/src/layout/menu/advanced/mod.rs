// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::Ui;

pub mod danger;

use crate::component::{Button, Heading, Identity, View};
use crate::theme::color::DARK_RED;
use crate::{AppState, Command, Menu, Stage};

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Advanced").render(ui);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
        }

        if Button::new("Deterministic entropy").render(ui).clicked() {
            app.stage = Stage::Command(Command::DeterministicEntropy);
        }
        ui.add_space(5.0);
        #[cfg(feature = "nostr")]
        {
            if Button::new("Nostr").render(ui).clicked() {
                app.set_stage(Stage::Menu(Menu::Nostr));
            }
            ui.add_space(5.0);
        }
        if Button::new("Danger")
            .background_color(DARK_RED)
            .render(ui)
            .clicked()
        {
            app.stage = Stage::Menu(Menu::Danger);
        }
        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.stage = Stage::Menu(Menu::Main);
        }
    });
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Layout, Ui};

pub mod danger;

use crate::component::{Button, Heading, Identity, Version};
use crate::theme::color::DARK_RED;
use crate::{AppState, Menu, Stage};

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.set_max_width(ui.available_width() - 20.0);

        Heading::new("Advanced").render(ui);

        ui.add_space(15.0);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
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

    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        Version::new().render(ui)
    });
}

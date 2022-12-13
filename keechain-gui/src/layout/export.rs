// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Layout, Ui};

use crate::component::{Button, Heading, Version};
use crate::{AppState, ExportTypes, Menu, Stage};

pub fn update_layout(app: &mut AppState, export_type: ExportTypes, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.set_max_width(ui.available_width() - 20.0);

        Heading::new("Export").render(ui);

        ui.add_space(15.0);

        match export_type {
            ExportTypes::Descriptors => {
                if Button::new("Back").render(ui).clicked() {
                    app.stage = Stage::Menu(Menu::Export);
                }
            }
            ExportTypes::BitcoinCore => {
                if Button::new("Back").render(ui).clicked() {
                    app.stage = Stage::Menu(Menu::Export);
                }
            }
            ExportTypes::Electrum => {
                if Button::new("Back").render(ui).clicked() {
                    app.stage = Stage::Menu(Menu::Export);
                }
            }
        }
    });

    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        Version::new().render(ui)
    });
}

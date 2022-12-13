// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Layout, Ui};
use eframe::Frame;
use keechain_core::util::bip::bip32::Bip32RootKey;

use crate::component::{Button, Heading, Version};
use crate::theme::color::DARK_RED;
use crate::{AppState, Command, ExportTypes, Menu, Stage};

pub fn update_layout(app: &mut AppState, menu: Menu, ui: &mut Ui, frame: &mut Frame) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.set_max_width(ui.available_width() - 20.0);

        Heading::new("Menu").render(ui);

        ui.add_space(15.0);

        if let Some(keechain) = &app.keechain {
            ui.group(|ui| {
                if let Ok(fingerprint) = keechain.keychain.seed.fingerprint(app.network) {
                    ui.label(format!("Fingerprint: {}", fingerprint));
                }
                ui.label(format!(
                    "Using passphrase: {}",
                    keechain.keychain.seed.passphrase().is_some()
                ));
            });
        }

        ui.add_space(15.0);

        match menu {
            Menu::Main => {
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
            }
            Menu::Export => {
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
            }
            Menu::Advanced => {
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
            }
            Menu::Setting => {
                if Button::new("Back").render(ui).clicked() {
                    app.stage = Stage::Menu(Menu::Main);
                }
            }
            Menu::Danger => {
                if Button::new("Back").render(ui).clicked() {
                    app.stage = Stage::Menu(Menu::Advanced);
                }
            }
        }
    });

    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        Version::new().render(ui)
    });
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use bdk::keys::bip39::Mnemonic;
use eframe::egui::{Align, Key, Layout, RichText, ScrollArea, Ui};
use eframe::epaint::Color32;

use crate::command;
use crate::gui::component::{Button, Heading, InputField, Version};
use crate::gui::theme::color::ORANGE;
use crate::gui::{AppState, Menu, Stage};

#[derive(Clone, Default)]
pub struct RestoreState {
    name: String,
    mnemonic: String,
    use_passphrase: bool,
    passphrase: Option<String>,
    password: String,
    confirm_password: String,
    error: Option<String>,
}

impl RestoreState {
    pub fn clear(&mut self) {
        self.name = String::new();
        self.mnemonic = String::new();
        self.use_passphrase = false;
        self.passphrase = None;
        self.password = String::new();
        self.confirm_password = String::new();
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.set_max_width(ui.available_width() - 20.0);

            Heading::new("Restore keychain").render(ui);

            ui.add_space(15.0);

            InputField::new("Name")
                .placeholder("Name of keychain")
                .render(ui, &mut app.layouts.restore.name);

            ui.add_space(7.0);

            InputField::new("Password")
                .placeholder("Password")
                .is_password()
                .render(ui, &mut app.layouts.restore.password);

            ui.add_space(7.0);

            InputField::new("Confirm password")
                .placeholder("Confirm password")
                .is_password()
                .render(ui, &mut app.layouts.restore.confirm_password);

            ui.add_space(7.0);

            InputField::new("Mnemonic (BIP39)")
                .placeholder("Mnemonic")
                .rows(5)
                .render(ui, &mut app.layouts.restore.mnemonic);

            ui.add_space(7.0);

            if app.layouts.restore.use_passphrase {
                if app.layouts.restore.passphrase.is_none() {
                    app.layouts.restore.passphrase = Some(String::new());
                }
                if let Some(passphrase) = app.layouts.restore.passphrase.as_mut() {
                    InputField::new("Passphrase (optional)")
                        .placeholder("Passphrase")
                        .render(ui, passphrase);
                }
            } else {
                app.layouts.restore.passphrase = None;
            }

            ui.add_space(5.0);

            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.checkbox(&mut app.layouts.restore.use_passphrase, "Use passphrase");
            });

            ui.add_space(7.0);

            if let Some(error) = &app.layouts.restore.error {
                ui.label(RichText::new(error).color(Color32::RED));
            }

            ui.add_space(15.0);

            let is_ready: bool = !app.layouts.restore.name.is_empty()
                && !app.layouts.restore.password.is_empty()
                && !app.layouts.restore.confirm_password.is_empty()
                && !app.layouts.restore.mnemonic.is_empty();

            let button = Button::new("Restore")
                .background_color(ORANGE)
                .enabled(is_ready)
                .render(ui);

            ui.add_space(5.0);

            if Button::new("Back").render(ui).clicked() {
                app.layouts.restore.clear();
                app.set_stage(Stage::Start);
            }

            if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
                if app.layouts.restore.password != app.layouts.restore.confirm_password {
                    app.layouts.restore.error = Some("Passwords not match".to_string());
                } else {
                    match Mnemonic::from_str(&app.layouts.restore.mnemonic) {
                        Ok(mnemonic) => match command::restore(
                            app.layouts.restore.name.clone(),
                            || Ok(app.layouts.restore.password.clone()),
                            || Ok(mnemonic),
                            || Ok(app.layouts.restore.passphrase.clone()),
                        ) {
                            Ok(seed) => {
                                app.layouts.restore.clear();
                                app.set_seed(Some(seed));
                                app.set_stage(Stage::Menu(Menu::Main));
                            }
                            Err(e) => app.layouts.restore.error = Some(e.to_string()),
                        },
                        Err(e) => app.layouts.restore.error = Some(e.to_string()),
                    }
                }
            }
        });

        ui.add_space(20.0);

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            Version::new().render(ui);
        });
    });
}

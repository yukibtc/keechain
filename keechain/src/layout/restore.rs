// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use eframe::egui::{Key, RichText, Ui};
use eframe::epaint::Color32;
use keechain_core::bips::bip39::Mnemonic;
use keechain_core::types::KeeChain;

use crate::component::{Button, Heading, InputField, View};
use crate::theme::color::ORANGE;
use crate::{AppState, Menu, Stage, KEYCHAINS_PATH};

#[derive(Default)]
pub struct RestoreState {
    name: String,
    mnemonic: String,
    password: String,
    confirm_password: String,
    error: Option<String>,
}

impl RestoreState {
    pub fn clear(&mut self) {
        self.name = String::new();
        self.mnemonic = String::new();
        self.password = String::new();
        self.confirm_password = String::new();
        self.error = None;
    }
}

pub fn update(app: &mut AppState, ui: &mut Ui) {
    View::show(ui, |ui| {
        Heading::new("Restore keychain").render(ui);

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

        if is_ready && (ui.input(|i| i.key_pressed(Key::Enter)) || button.clicked()) {
            if app.layouts.restore.password != app.layouts.restore.confirm_password {
                app.layouts.restore.error = Some("Passwords not match".to_string());
            } else {
                match Mnemonic::from_str(&app.layouts.restore.mnemonic) {
                    Ok(mnemonic) => match KeeChain::restore(
                        KEYCHAINS_PATH.as_path(),
                        app.layouts.restore.name.clone(),
                        || Ok(app.layouts.restore.password.clone()),
                        || Ok(mnemonic),
                    ) {
                        Ok(keechain) => {
                            app.layouts.restore.clear();
                            app.set_keechain(Some(keechain));
                            app.set_stage(Stage::Menu(Menu::Main));
                        }
                        Err(e) => app.layouts.restore.error = Some(e.to_string()),
                    },
                    Err(e) => app.layouts.restore.error = Some(e.to_string()),
                }
            }
        }
    });
}

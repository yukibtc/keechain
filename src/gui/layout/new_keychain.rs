// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Key, Layout, RichText, ScrollArea, Ui};
use eframe::epaint::Color32;

use crate::command;
use crate::core::types::{Seed, WordCount};
use crate::gui::component::{Button, Heading, InputField, Version};
use crate::gui::theme::color::ORANGE;
use crate::gui::{AppState, Menu, Stage};

#[derive(Clone, Default)]
pub struct NewKeychainState {
    name: String,
    use_passphrase: bool,
    passphrase: Option<String>,
    password: String,
    confirm_password: String,
    seed: Option<Seed>,
    confirm_saved_mnemonic: bool,
    error: Option<String>,
}

impl NewKeychainState {
    pub fn clear(&mut self) {
        self.name = String::new();
        self.use_passphrase = false;
        self.passphrase = None;
        self.password = String::new();
        self.confirm_password = String::new();
        self.seed = None;
        self.confirm_saved_mnemonic = false;
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.set_max_width(ui.available_width() - 20.0);

            Heading::new("Generate keychain").render(ui);

            ui.add_space(15.0);

            if let Some(seed) = app.layouts.new_keychain.seed.clone() {
                show_mnemonic_layout(app, seed, ui);
            } else {
                generate_layout(app, ui);
            }
        });

        ui.add_space(20.0);

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            Version::new().render(ui);
        });
    });
}

fn generate_layout(app: &mut AppState, ui: &mut Ui) {
    InputField::new("Name")
        .placeholder("Name of keychain")
        .render(ui, &mut app.layouts.new_keychain.name);

    ui.add_space(7.0);

    InputField::new("Password")
        .placeholder("Password")
        .is_password()
        .render(ui, &mut app.layouts.new_keychain.password);

    ui.add_space(7.0);

    InputField::new("Confirm password")
        .placeholder("Confirm password")
        .is_password()
        .render(ui, &mut app.layouts.new_keychain.confirm_password);

    ui.add_space(7.0);

    if app.layouts.new_keychain.use_passphrase {
        if app.layouts.new_keychain.passphrase.is_none() {
            app.layouts.new_keychain.passphrase = Some(String::new());
        }
        if let Some(passphrase) = app.layouts.new_keychain.passphrase.as_mut() {
            InputField::new("Passphrase (optional)")
                .placeholder("Passphrase")
                .render(ui, passphrase);
        }
    } else {
        app.layouts.new_keychain.passphrase = None;
    }

    ui.add_space(5.0);

    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.checkbox(
            &mut app.layouts.new_keychain.use_passphrase,
            "Use passphrase",
        );
    });

    ui.add_space(7.0);

    if let Some(error) = &app.layouts.new_keychain.error {
        ui.label(RichText::new(error).color(Color32::RED));
    }

    ui.add_space(15.0);

    let is_ready: bool = !app.layouts.new_keychain.name.is_empty()
        && !app.layouts.new_keychain.password.is_empty()
        && !app.layouts.new_keychain.confirm_password.is_empty()
        && app.layouts.new_keychain.seed.is_none();

    let button = Button::new("Generate")
        .background_color(ORANGE)
        .enabled(is_ready)
        .render(ui);

    ui.add_space(5.0);

    if Button::new("Back").render(ui).clicked() {
        app.layouts.new_keychain.clear();
        app.set_stage(Stage::Start);
    }

    if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
        if app.layouts.new_keychain.password != app.layouts.new_keychain.confirm_password {
            app.layouts.new_keychain.error = Some("Passwords not match".to_string());
        } else {
            match command::generate(
                app.layouts.new_keychain.name.clone(),
                || Ok(app.layouts.new_keychain.password.clone()),
                || Ok(app.layouts.new_keychain.passphrase.clone()),
                WordCount::W24,
                || Ok(None),
            ) {
                Ok(seed) => {
                    app.layouts.new_keychain.seed = Some(seed);
                }
                Err(e) => app.layouts.new_keychain.error = Some(e.to_string()),
            }
        }
    }
}

fn show_mnemonic_layout(app: &mut AppState, seed: Seed, ui: &mut Ui) {
    ui.add_space(25.0);

    ui.label(RichText::new(seed.mnemonic().to_string()).monospace());

    ui.add_space(25.0);

    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.checkbox(
            &mut app.layouts.new_keychain.confirm_saved_mnemonic,
            "I saved the mnemonic in a secure and offline place",
        );
    });

    ui.add_space(15.0);

    let button = Button::new("Open keychain")
        .background_color(ORANGE)
        .enabled(app.layouts.new_keychain.confirm_saved_mnemonic)
        .render(ui);

    if button.clicked() {
        app.set_seed(Some(seed));
        app.layouts.restore.clear();
        app.set_stage(Stage::Menu(Menu::Main));
    }

    ui.add_space(5.0);

    if Button::new("Back").render(ui).clicked() {
        app.layouts.new_keychain.clear();
        app.set_stage(Stage::Menu(Menu::Main));
    }
}

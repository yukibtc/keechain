// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Key, Layout, RichText, Ui};
use eframe::epaint::Color32;

use crate::component::{Button, Heading, Identity, InputField, View};
use crate::theme::color::{DARK_RED, ORANGE};
use crate::{AppState, Menu, Stage};

#[derive(Default)]
pub struct PassphraseState {
    passphrase: String,
    save: bool,
    show_saved: bool,
    error: Option<String>,
}

impl PassphraseState {
    pub fn clear(&mut self) {
        self.passphrase = String::new();
        self.save = false;
        self.show_saved = false;
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    View::show(ui, |ui| {
        Heading::new("Passphrase").render(ui);

        ui.add_space(15.0);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
        }

        if app.layouts.passphrase.show_saved {
            show_saved_layout(app, ui);
        } else {
            apply_new_layout(app, ui);
        }
    });
}

pub fn apply_new_layout(app: &mut AppState, ui: &mut Ui) {
    InputField::new("Passphrase")
        .placeholder("Passphrase")
        .render(ui, &mut app.layouts.passphrase.passphrase);

    ui.add_space(7.0);

    if let Some(error) = &app.layouts.passphrase.error {
        ui.label(RichText::new(error).color(Color32::RED));
    }

    ui.add_space(7.0);

    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.checkbox(
            &mut app.layouts.passphrase.save,
            "Save passphrase to keychain",
        );
    });

    ui.add_space(15.0);

    let is_ready: bool = !app.layouts.passphrase.passphrase.is_empty();

    let button = Button::new("Apply")
        .background_color(ORANGE)
        .enabled(is_ready)
        .render(ui);

    ui.add_space(5.0);

    if let Some(keechain) = app.keechain.as_mut() {
        if Button::new("Clear applied")
            .enabled(keechain.keychain.seed.passphrase().is_some())
            .background_color(DARK_RED)
            .render(ui)
            .clicked()
        {
            keechain.keychain.apply_passphrase::<String>(None);
            app.layouts.passphrase.clear();
            app.set_stage(Stage::Menu(Menu::Main));
        }
    }

    ui.add_space(5.0);

    if Button::new("Saved").render(ui).clicked() {
        app.layouts.passphrase.show_saved = true;
    }

    ui.add_space(5.0);

    if Button::new("Back").render(ui).clicked() {
        app.layouts.passphrase.clear();
        app.set_stage(Stage::Menu(Menu::Main));
    }

    if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
        match app.keechain.as_mut() {
            Some(keechain) => {
                if app.layouts.passphrase.save {
                    keechain
                        .keychain
                        .add_passphrase(app.layouts.passphrase.passphrase.clone());
                    if let Err(e) = keechain.save() {
                        app.layouts.passphrase.error = Some(e.to_string());
                    } else {
                        keechain
                            .keychain
                            .apply_passphrase(Some(app.layouts.passphrase.passphrase.clone()));
                        app.layouts.passphrase.clear();
                        app.set_stage(Stage::Menu(Menu::Main));
                    }
                } else {
                    keechain
                        .keychain
                        .apply_passphrase(Some(app.layouts.passphrase.passphrase.clone()));
                    app.layouts.passphrase.clear();
                    app.set_stage(Stage::Menu(Menu::Main));
                }
            }
            None => app.layouts.passphrase.error = Some("Impossible to get keechain".to_string()),
        }
    }
}

pub fn show_saved_layout(app: &mut AppState, ui: &mut Ui) {
    match app.keechain.as_mut() {
        Some(keechain) => {
            for passphrase in keechain.keychain.passphrases().iter() {
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut app.layouts.passphrase.passphrase,
                        passphrase.clone(),
                        passphrase,
                    );
                });
                ui.add_space(5.0);
            }
        }
        None => app.layouts.passphrase.error = Some("Impossible to get keechain".to_string()),
    }

    ui.add_space(2.0);

    if let Some(error) = &app.layouts.passphrase.error {
        ui.label(RichText::new(error).color(Color32::RED));
    }

    ui.add_space(15.0);

    let is_ready: bool = !app.layouts.passphrase.passphrase.is_empty();

    let button = Button::new("Apply")
        .background_color(ORANGE)
        .enabled(is_ready)
        .render(ui);

    ui.add_space(5.0);

    if Button::new("Back").render(ui).clicked() {
        app.layouts.passphrase.clear();
    }

    if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
        match app.keechain.as_mut() {
            Some(keechain) => {
                keechain
                    .keychain
                    .apply_passphrase(Some(app.layouts.passphrase.passphrase.clone()));
                app.layouts.passphrase.clear();
                app.set_stage(Stage::Menu(Menu::Main));
            }
            None => app.layouts.passphrase.error = Some("Impossible to get keechain".to_string()),
        }
    }
}

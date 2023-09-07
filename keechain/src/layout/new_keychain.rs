// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, ComboBox, Key, Layout, Ui};
use keechain_core::types::{KeeChain, WordCount};

use crate::component::{Button, Error, Heading, InputField, MnemonicViewer, View};
use crate::theme::color::ORANGE;
use crate::{AppState, Menu, Stage, KEYCHAINS_PATH};

const WORD_COUNT_OPTIONS: [WordCount; 3] = [WordCount::W12, WordCount::W18, WordCount::W24];

#[derive(Default)]
pub struct NewKeychainState {
    name: String,
    password: String,
    confirm_password: String,
    word_count: WordCount,
    keechain: Option<KeeChain>,
    confirm_saved_mnemonic: bool,
    error: Option<String>,
}

impl NewKeychainState {
    pub fn clear(&mut self) {
        self.name = String::new();
        self.password = String::new();
        self.confirm_password = String::new();
        self.word_count = WordCount::default();
        self.keechain = None;
        self.confirm_saved_mnemonic = false;
        self.error = None;
    }
}

pub fn update(app: &mut AppState, ui: &mut Ui) {
    View::show(ui, |ui| {
        Heading::new("Generate keychain").render(ui);

        if let Some(keechain) = app.layouts.new_keychain.keechain.clone() {
            show_mnemonic_layout(app, keechain, ui);
        } else {
            generate_layout(app, ui);
        }
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

    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.add_space(1.0);
        ui.label("Word count");
        ui.horizontal_wrapped(|ui| {
            ComboBox::from_id_source("word_count")
                .width(ui.available_width())
                .selected_text(app.layouts.new_keychain.word_count.as_u32().to_string())
                .show_ui(ui, |ui| {
                    for value in WORD_COUNT_OPTIONS.into_iter() {
                        ui.selectable_value(
                            &mut app.layouts.new_keychain.word_count,
                            value,
                            value.as_u32().to_string(),
                        );
                    }
                });
        })
    });

    ui.add_space(7.0);

    if let Some(error) = &app.layouts.new_keychain.error {
        Error::new(error).render(ui);
    }

    ui.add_space(15.0);

    let is_ready: bool = !app.layouts.new_keychain.name.is_empty()
        && !app.layouts.new_keychain.password.is_empty()
        && !app.layouts.new_keychain.confirm_password.is_empty()
        && app.layouts.new_keychain.keechain.is_none();

    let button = Button::new("Generate")
        .background_color(ORANGE)
        .enabled(is_ready)
        .render(ui);

    ui.add_space(5.0);

    if Button::new("Back").render(ui).clicked() {
        app.layouts.new_keychain.clear();
        app.set_stage(Stage::Start);
    }

    if is_ready && (ui.input(|i| i.key_pressed(Key::Enter)) || button.clicked()) {
        match KeeChain::generate(
            KEYCHAINS_PATH.as_path(),
            app.layouts.new_keychain.name.clone(),
            || Ok(app.layouts.new_keychain.password.clone()),
            || Ok(app.layouts.new_keychain.confirm_password.clone()),
            app.layouts.new_keychain.word_count,
            || Ok(None),
        ) {
            Ok(keechain) => {
                app.layouts.new_keychain.keechain = Some(keechain);
            }
            Err(e) => app.layouts.new_keychain.error = Some(e.to_string()),
        }
    }
}

fn show_mnemonic_layout(app: &mut AppState, keechain: KeeChain, ui: &mut Ui) {
    MnemonicViewer::new(keechain.keychain.seed.mnemonic()).render(ui);

    ui.add_space(7.0);

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
        app.set_keechain(Some(keechain));
        app.layouts.restore.clear();
        app.set_stage(Stage::Menu(Menu::Main));
    }

    ui.add_space(5.0);

    if Button::new("Back").render(ui).clicked() {
        app.layouts.new_keychain.clear();
        app.set_stage(Stage::Menu(Menu::Main));
    }
}

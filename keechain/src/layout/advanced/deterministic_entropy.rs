// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use eframe::egui::{Align, ComboBox, Key, Layout, Ui};
use keechain_core::bips::bip39::Mnemonic;
use keechain_core::types::{Index, WordCount};

use crate::component::{Button, Error, Heading, InputField, MnemonicViewer, View};
use crate::theme::color::ORANGE;
use crate::{AppState, Menu, Stage, SECP256K1};

const WORD_COUNT_OPTIONS: [WordCount; 3] = [WordCount::W12, WordCount::W18, WordCount::W24];

#[derive(Default)]
pub struct DeterministicEntropyState {
    word_count: WordCount,
    index: String,
    mnemonic: Option<Mnemonic>,
    error: Option<String>,
}

impl DeterministicEntropyState {
    pub fn clear(&mut self) {
        self.word_count = WordCount::W24;
        self.index = String::new();
        self.mnemonic = None;
        self.error = None;
    }
}

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Deterministic entropy (BIP85)").render(ui);

        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.add_space(1.0);
            ui.label("Word count");
            ui.horizontal_wrapped(|ui| {
                ComboBox::from_id_source("word_count")
                    .width(ui.available_width())
                    .selected_text(
                        app.layouts
                            .deterministic_entropy
                            .word_count
                            .as_u32()
                            .to_string(),
                    )
                    .show_ui(ui, |ui| {
                        for value in WORD_COUNT_OPTIONS.into_iter() {
                            ui.selectable_value(
                                &mut app.layouts.deterministic_entropy.word_count,
                                value,
                                value.as_u32().to_string(),
                            );
                        }
                    });
            })
        });

        ui.add_space(7.0);

        InputField::new("Index")
            .placeholder("Index (between 0 and 2^31 - 1)")
            .render(ui, &mut app.layouts.deterministic_entropy.index);

        ui.add_space(7.0);

        if let Some(mnemonic) = app.layouts.deterministic_entropy.mnemonic.as_ref() {
            MnemonicViewer::new(mnemonic.clone()).render(ui);
            ui.add_space(7.0);
        }

        if let Some(error) = &app.layouts.deterministic_entropy.error {
            Error::new(error).render(ui);
        }

        ui.add_space(15.0);

        let is_ready: bool = !app.layouts.deterministic_entropy.index.is_empty();

        let button = Button::new("Derive")
            .background_color(ORANGE)
            .enabled(is_ready)
            .render(ui);

        if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
            match app.keechain.as_mut() {
                Some(keechain) => {
                    match Index::from_str(app.layouts.deterministic_entropy.index.as_str()) {
                        Ok(index) => match keechain.keychain.deterministic_entropy(
                            app.layouts.deterministic_entropy.word_count,
                            index,
                            &SECP256K1,
                        ) {
                            Ok(mnemonic) => {
                                app.layouts.deterministic_entropy.error = None;
                                app.layouts.deterministic_entropy.mnemonic = Some(mnemonic);
                            }
                            Err(e) => app.layouts.deterministic_entropy.error = Some(e.to_string()),
                        },
                        Err(e) => app.layouts.deterministic_entropy.error = Some(e.to_string()),
                    }
                }
                None => {
                    app.layouts.deterministic_entropy.error =
                        Some("Impossible to get keechain".to_string())
                }
            }
        }

        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.layouts.deterministic_entropy.clear();
            app.stage = Stage::Menu(Menu::Advanced);
        }
    });
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::Path;

use eframe::egui::{Key, Ui};
use keechain_core::util::dir;

use crate::component::{Button, Error, Heading, InputField, View};
use crate::theme::color::ORANGE;
use crate::{AppState, Menu, Stage, KEYCHAINS_PATH};

#[derive(Default)]
pub struct RenameKeychainState {
    new_name: String,
    error: Option<String>,
}

impl RenameKeychainState {
    pub fn clear(&mut self) {
        self.new_name = String::new();
        self.error = None;
    }
}

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Rename keychain").render(ui);

        InputField::new("Name")
            .placeholder("New name of keychain")
            .render(ui, &mut app.layouts.rename_keychain.new_name);

        ui.add_space(7.0);

        if let Some(error) = &app.layouts.rename_keychain.error {
            Error::new(error).render(ui);
        }

        ui.add_space(15.0);

        let is_ready: bool = !app.layouts.rename_keychain.new_name.is_empty();
        let button = Button::new("Rename")
            .background_color(ORANGE)
            .enabled(is_ready)
            .render(ui);

        if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
            match app.keechain.as_mut() {
                Some(keechain) => {
                    match dir::get_keychain_file::<&Path, String>(
                        KEYCHAINS_PATH.as_ref(),
                        app.layouts.rename_keychain.new_name.clone(),
                    ) {
                        Ok(path) => match keechain.rename(path) {
                            Ok(_) => {
                                app.layouts.rename_keychain.clear();
                                app.stage = Stage::Menu(Menu::Setting);
                            }
                            Err(e) => app.layouts.rename_keychain.error = Some(e.to_string()),
                        },
                        Err(e) => app.layouts.rename_keychain.error = Some(e.to_string()),
                    }
                }
                None => {
                    app.layouts.rename_keychain.error =
                        Some("Impossible to get keechain".to_string())
                }
            }
        }

        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.layouts.rename_keychain.clear();
            app.stage = Stage::Menu(Menu::Setting);
        }
    });
}

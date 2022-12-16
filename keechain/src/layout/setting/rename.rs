// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Key, RichText, Ui};

use crate::component::{Button, Heading, InputField, View};
use crate::theme::color::{ORANGE, RED};
use crate::{AppState, Menu, Stage};

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

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
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
            ui.label(RichText::new(error).color(RED));
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
                    match keechain.rename(app.layouts.rename_keychain.new_name.clone()) {
                        Ok(_) => {
                            app.layouts.rename_keychain.clear();
                            app.stage = Stage::Menu(Menu::Setting);
                        }
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

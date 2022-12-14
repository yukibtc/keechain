// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Key, Layout, RichText, Ui};

use crate::component::{Button, Heading, InputField, Version};
use crate::theme::color::{ORANGE, RED};
use crate::{AppState, Menu, Stage};

#[derive(Default)]
pub struct WipeKeychainState {
    password: String,
    error: Option<String>,
}

impl WipeKeychainState {
    pub fn clear(&mut self) {
        self.password = String::new();
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.set_max_width(ui.available_width() - 20.0);

        Heading::new("Delete keychain").render(ui);

        ui.add_space(15.0);

        InputField::new("Password")
            .placeholder("Password")
            .is_password()
            .render(ui, &mut app.layouts.wipe_keychain.password);

        ui.add_space(7.0);

        if let Some(error) = &app.layouts.wipe_keychain.error {
            ui.label(RichText::new(error).color(RED));
        }

        ui.add_space(15.0);

        let is_ready: bool = !app.layouts.wipe_keychain.password.is_empty();

        let button = Button::new("Delete")
            .background_color(ORANGE)
            .enabled(is_ready)
            .render(ui);

        if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
            match app.keechain.as_mut() {
                Some(keechain) => {
                    if keechain.check_password(app.layouts.wipe_keychain.password.clone()) {
                        match keechain.wipe() {
                            Ok(_) => {
                                app.layouts.wipe_keychain.clear();
                                app.set_keechain(None);
                                app.set_stage(Stage::Start);
                            }
                            Err(e) => app.layouts.wipe_keychain.error = Some(e.to_string()),
                        }
                    } else {
                        app.layouts.wipe_keychain.error = Some("Wrong password".to_string())
                    }
                }
                None => {
                    app.layouts.wipe_keychain.error = Some("Impossible to get keechain".to_string())
                }
            }
        }

        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.layouts.wipe_keychain.clear();
            app.stage = Stage::Menu(Menu::Danger);
        }
    });

    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        Version::new().render(ui)
    });
}

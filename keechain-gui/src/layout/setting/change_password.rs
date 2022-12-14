// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Key, RichText, Ui};

use crate::component::{Button, Heading, InputField, View};
use crate::theme::color::{ORANGE, RED};
use crate::{AppState, Menu, Stage};

#[derive(Default)]
pub struct ChangePasswordState {
    current_password: String,
    new_password: String,
    confirm_new_password: String,
    error: Option<String>,
}

impl ChangePasswordState {
    pub fn clear(&mut self) {
        self.current_password = String::new();
        self.new_password = String::new();
        self.confirm_new_password = String::new();
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Change password").render(ui);

        ui.add_space(15.0);

        InputField::new("Current password")
            .placeholder("Current password")
            .is_password()
            .render(ui, &mut app.layouts.change_password.current_password);

        ui.add_space(7.0);

        InputField::new("New password")
            .placeholder("New password")
            .is_password()
            .render(ui, &mut app.layouts.change_password.new_password);

        ui.add_space(7.0);

        InputField::new("Confirm new password")
            .placeholder("Confirm new password")
            .is_password()
            .render(ui, &mut app.layouts.change_password.confirm_new_password);

        ui.add_space(7.0);

        if let Some(error) = &app.layouts.change_password.error {
            ui.label(RichText::new(error).color(RED));
        }

        ui.add_space(15.0);

        let is_ready: bool = !app.layouts.change_password.new_password.is_empty()
            && !app.layouts.change_password.confirm_new_password.is_empty();

        let button = Button::new("Rename")
            .background_color(ORANGE)
            .enabled(is_ready)
            .render(ui);

        if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
            if app.layouts.change_password.new_password
                != app.layouts.change_password.confirm_new_password
            {
                app.layouts.change_password.error = Some("Passwords not match".to_string());
            } else {
                match app.keechain.as_mut() {
                    Some(keechain) => {
                        if keechain
                            .check_password(app.layouts.change_password.current_password.clone())
                        {
                            match keechain.change_password(|| {
                                Ok(app.layouts.change_password.new_password.clone())
                            }) {
                                Ok(_) => {
                                    app.layouts.change_password.clear();
                                    app.stage = Stage::Menu(Menu::Setting);
                                }
                                Err(e) => app.layouts.change_password.error = Some(e.to_string()),
                            }
                        } else {
                            app.layouts.change_password.error =
                                Some("Current password is wrong".to_string())
                        }
                    }
                    None => {
                        app.layouts.change_password.error =
                            Some("Impossible to get keechain".to_string())
                    }
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

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use bdk::keys::bip39::Mnemonic;
use eframe::egui::{Align, CentralPanel, Context, Key, Layout, RichText, ScrollArea, TextEdit};
use eframe::epaint::Color32;

use crate::command;
use crate::gui::component::{Button, Heading, InputField, Version};
use crate::gui::{AppData, AppStage, Menu};

#[derive(Clone, Default)]
pub struct RestoreLayoutData {
    name: String,
    mnemonic: String,
    use_passphrase: bool,
    passphrase: Option<String>,
    password: String,
    confirm_password: String,
    error: Option<String>,
}

impl RestoreLayoutData {
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

pub fn update_layout(app: &mut AppData, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                Heading::new("Restore keychain").render(ui);

                ui.add_space(15.0);

                InputField::new("Name")
                    .render(ui, TextEdit::singleline(&mut app.layouts.restore.name));

                ui.add_space(7.0);

                InputField::new("Password").render(
                    ui,
                    TextEdit::singleline(&mut app.layouts.restore.password).password(true),
                );

                ui.add_space(7.0);

                InputField::new("Confirm password").render(
                    ui,
                    TextEdit::singleline(&mut app.layouts.restore.confirm_password).password(true),
                );

                ui.add_space(7.0);

                InputField::new("Mnemonic (BIP39)").render(
                    ui,
                    TextEdit::multiline(&mut app.layouts.restore.mnemonic).desired_rows(5),
                );

                ui.add_space(7.0);

                if app.layouts.restore.use_passphrase {
                    if app.layouts.restore.passphrase.is_none() {
                        app.layouts.restore.passphrase = Some(String::new());
                    }
                    if let Some(passphrase) = app.layouts.restore.passphrase.as_mut() {
                        InputField::new("Passphrase (optional)")
                            .render(ui, TextEdit::singleline(passphrase));
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

                ui.add_space(10.0);

                let is_ready: bool = !app.layouts.restore.name.is_empty()
                    && !app.layouts.restore.password.is_empty()
                    && !app.layouts.restore.confirm_password.is_empty()
                    && !app.layouts.restore.mnemonic.is_empty();

                let button = Button::new("Restore").enabled(is_ready).render(ui);

                ui.add_space(5.0);

                if Button::new("Back").render(ui).clicked() {
                    app.layouts.restore.clear();
                    app.set_stage(AppStage::Start);
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
                                    app.set_stage(AppStage::Menu(Menu::Main));
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
    });
}

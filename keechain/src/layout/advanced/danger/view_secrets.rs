// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Key, Ui};
use keechain_core::types::Secrets;

use crate::component::{Button, Error, Heading, InputField, MnemonicViewer, ReadOnlyField, View};
use crate::theme::color::ORANGE;
use crate::{AppState, Menu, Stage};

#[derive(Default)]
pub struct ViewSecretsState {
    password: String,
    secrets: Option<Secrets>,
    error: Option<String>,
}

impl ViewSecretsState {
    pub fn clear(&mut self) {
        self.password = String::new();
        self.secrets = None;
        self.error = None;
    }
}

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("View secrets").render(ui);

        if let Some(secrets) = &app.layouts.view_secrets.secrets {
            ReadOnlyField::new("Entropy", &secrets.entropy)
                .rows(2)
                .render(ui);
            ui.add_space(5.0);
            MnemonicViewer::new(secrets.mnemonic.clone()).render(ui);
            if let Some(passphrase) = secrets.passphrase.as_ref() {
                ui.add_space(5.0);
                ReadOnlyField::new("Passphrase", passphrase)
                    .rows(1)
                    .render(ui);
            }
            ui.add_space(10.0);
        } else {
            InputField::new("Password")
                .placeholder("Password")
                .is_password()
                .render(ui, &mut app.layouts.view_secrets.password);

            ui.add_space(7.0);

            if let Some(error) = &app.layouts.view_secrets.error {
                Error::new(error).render(ui);
            }

            ui.add_space(15.0);

            let is_ready: bool = !app.layouts.view_secrets.password.is_empty();

            let button = Button::new("View")
                .background_color(ORANGE)
                .enabled(is_ready)
                .render(ui);

            if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
                match app.keechain.as_mut() {
                    Some(keechain) => {
                        if keechain.check_password(app.layouts.view_secrets.password.clone()) {
                            match keechain.keychain.secrets(app.network) {
                                Ok(secrets) => app.layouts.view_secrets.secrets = Some(secrets),
                                Err(e) => app.layouts.view_secrets.error = Some(e.to_string()),
                            }
                        } else {
                            app.layouts.view_secrets.error = Some("Wrong password".to_string())
                        }
                    }
                    None => {
                        app.layouts.view_secrets.error =
                            Some("Impossible to get keechain".to_string())
                    }
                }
            }
        }

        ui.add_space(5.0);
        if Button::new("Back").render(ui).clicked() {
            app.layouts.view_secrets.clear();
            app.stage = Stage::Menu(Menu::Danger);
        }
    });
}

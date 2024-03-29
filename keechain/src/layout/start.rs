// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::Path;
use std::sync::Arc;

use eframe::egui::{self, Align, ComboBox, Key, Layout, Ui};
use egui_extras::RetainedImage;
use keechain_core::types::KeeChain;
use keechain_core::util::dir;

use crate::component::{Button, Error, InputField, View};
use crate::theme::color::ORANGE;
use crate::{AppState, Menu, Stage, KEYCHAINS_PATH, SECP256K1};

const LOGO: &[u8] = include_bytes!("../../assets/logo.png");

pub struct StartState {
    name: String,
    password: String,
    error: Option<String>,
    logo: Arc<RetainedImage>,
}

impl Default for StartState {
    fn default() -> Self {
        Self {
            name: String::new(),
            password: String::new(),
            error: None,
            logo: Arc::new(
                RetainedImage::from_image_bytes("logo.png", LOGO).expect("Impossible to load logo"),
            ),
        }
    }
}

impl StartState {
    pub fn clear(&mut self) {
        self.name = String::new();
        self.password = String::new();
        self.error = None;
    }
}

pub fn update(app: &mut AppState, ui: &mut Ui) {
    View::show(ui, |ui| {
        ui.add_space(25.0);

        app.layouts
            .start
            .logo
            .show_size(ui, egui::vec2(175.0, 175.0));

        ui.add_space(5.0);

        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.add_space(1.0);
            ui.label("Keychain");
            ui.horizontal_wrapped(|ui| {
                ComboBox::from_id_source("name")
                    .width(ui.available_width())
                    .selected_text(if app.layouts.start.name.is_empty() {
                        "Select keychain"
                    } else {
                        app.layouts.start.name.as_str()
                    })
                    .show_ui(ui, |ui| {
                        if let Ok(list) = dir::get_keychains_list::<&Path>(KEYCHAINS_PATH.as_ref())
                        {
                            for value in list.into_iter() {
                                ui.selectable_value(
                                    &mut app.layouts.start.name,
                                    value.clone(),
                                    value.as_str(),
                                );
                            }
                        }
                    });
            })
        });

        ui.add_space(7.0);

        InputField::new("Password")
            .placeholder("Password")
            .is_password()
            .render(ui, &mut app.layouts.start.password);

        ui.add_space(7.0);

        if let Some(error) = &app.layouts.start.error {
            Error::new(error).render(ui);
        }

        ui.add_space(15.0);

        let is_ready: bool =
            !app.layouts.start.name.is_empty() && !app.layouts.start.password.is_empty();
        let button = Button::new("Open")
            .background_color(ORANGE)
            .enabled(is_ready)
            .render(ui);

        ui.add_space(7.0);
        ui.separator();
        ui.add_space(7.0);

        if Button::new("Create a new keychain").render(ui).clicked() {
            app.layouts.start.clear();
            app.set_stage(Stage::NewKeychain);
        }

        ui.add_space(5.0);

        if Button::new("Restore").render(ui).clicked() {
            app.layouts.start.clear();
            app.set_stage(Stage::RestoreKeychain);
        }

        if is_ready && (ui.input(|i| i.key_pressed(Key::Enter)) || button.clicked()) {
            match KeeChain::open(
                KEYCHAINS_PATH.as_path(),
                app.layouts.start.name.clone(),
                || Ok(app.layouts.start.password.clone()),
                app.network,
                &SECP256K1,
            ) {
                Ok(keechain) => {
                    app.layouts.start.clear();
                    app.set_keechain(Some(keechain));
                    app.set_stage(Stage::Menu(Menu::Main));
                }
                Err(e) => app.layouts.start.error = Some(e.to_string()),
            }
        }
    });
}

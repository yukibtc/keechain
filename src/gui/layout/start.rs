// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, CentralPanel, ComboBox, Context, Key, Layout, RichText, TextEdit};
use eframe::epaint::Color32;

use crate::command;
use crate::core::util::dir;
use crate::gui::component::{Button, Heading, InputField, Version};
use crate::gui::theme::color::BITCOIN_ORANGE;
use crate::gui::{AppData, AppStage, Menu};

#[derive(Clone, Default)]
pub struct StartLayoutData {
    name: String,
    password: String,
    error: Option<String>,
}

impl StartLayoutData {
    pub fn clear(&mut self) {
        self.name = String::new();
        self.password = String::new();
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppData, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            Heading::new("Open keychain").render(ui);

            ui.add_space(15.0);

            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.add_space(1.0);
                ui.label("Keychain");
                ui.horizontal_wrapped(|ui| {
                    ComboBox::from_id_source("name")
                        .width(ui.available_width() - 10.0)
                        .selected_text(if app.layouts.start.name.is_empty() {
                            "Select keychain"
                        } else {
                            app.layouts.start.name.as_str()
                        })
                        .show_ui(ui, |ui| {
                            if let Ok(list) = dir::get_keychains_list() {
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

            InputField::new("Password").render(
                ui,
                TextEdit::singleline(&mut app.layouts.start.password)
                    .hint_text("Password")
                    .password(true),
            );

            ui.add_space(7.0);

            if let Some(error) = &app.layouts.start.error {
                ui.label(RichText::new(error).color(Color32::RED));
            }

            ui.add_space(25.0);

            let is_ready: bool =
                !app.layouts.start.name.is_empty() && !app.layouts.start.password.is_empty();
            let button = Button::new("Open")
                .background_color(BITCOIN_ORANGE)
                .enabled(is_ready)
                .render(ui);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            if Button::new("Create a new keychain").render(ui).clicked() {
                app.set_stage(AppStage::NewKeychain);
            }

            ui.add_space(5.0);

            if Button::new("Restore").render(ui).clicked() {
                app.set_stage(AppStage::RestoreKeychain);
            }

            if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
                match command::open(app.layouts.start.name.clone(), || {
                    Ok(app.layouts.start.password.clone())
                }) {
                    Ok(seed) => {
                        app.layouts.start.clear();
                        app.set_seed(Some(seed));
                        app.set_stage(AppStage::Menu(Menu::Main));
                    }
                    Err(e) => app.layouts.start.error = Some(e.to_string()),
                }
            }
        });

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            Version::new().render(ui)
        });
    });
}

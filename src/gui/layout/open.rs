// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{
    Align, Button, CentralPanel, ComboBox, Context, Key, Layout, RichText, TextEdit,
};
use eframe::epaint::Color32;

use crate::command;
use crate::gui::{AppData, AppStage, Menu};
use crate::util::dir;

#[derive(Clone, Default)]
pub struct OpenLayout {
    name: String,
    password: String,
    error: Option<String>,
}

impl OpenLayout {
    pub fn clear(&mut self) {
        self.name = String::new();
        self.password = String::new();
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppData, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.heading("Open keychain");

            ui.add_space(15.0);

            ui.horizontal(|ui| {
                ui.label("Keychain");
                ComboBox::from_id_source("name")
                    .width(ui.available_width() - 10.0)
                    .selected_text(if app.open_layout.name.is_empty() {
                        "Select keychain"
                    } else {
                        app.open_layout.name.as_str()
                    })
                    .show_ui(ui, |ui| {
                        if let Ok(list) = dir::get_keychains_list() {
                            for value in list.into_iter() {
                                ui.selectable_value(
                                    &mut app.open_layout.name,
                                    value.clone(),
                                    value.as_str(),
                                );
                            }
                        }
                    });
            });

            ui.add_space(7.0);
            ui.horizontal(|ui| {
                ui.label("Password");
                ui.add_sized(
                    [ui.available_width(), 18.0],
                    TextEdit::singleline(&mut app.open_layout.password).password(true),
                );
            });
            ui.add_space(7.0);
            if let Some(error) = &app.open_layout.error {
                ui.label(RichText::new(error).color(Color32::RED));
            }
            ui.add_space(10.0);

            let is_ready: bool =
                !app.open_layout.name.is_empty() && !app.open_layout.password.is_empty();
            let mut button = ui.add_enabled(is_ready, Button::new("Unlock"));
            button.rect.min.x = 100.0;
            button.rect.max.x = 100.0;

            if is_ready && (ui.input().key_pressed(Key::Enter) || button.clicked()) {
                match command::open(app.open_layout.name.clone(), || {
                    Ok(app.open_layout.password.clone())
                }) {
                    Ok(seed) => {
                        app.open_layout.clear();
                        app.set_seed(Some(seed));
                        app.set_stage(AppStage::Menu(Menu::Main));
                    }
                    Err(e) => app.open_layout.error = Some(e.to_string()),
                }
            }
        });
    });
}

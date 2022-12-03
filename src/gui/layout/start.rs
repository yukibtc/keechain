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
            ui.heading("Open keychain");

            ui.add_space(15.0);

            ui.horizontal(|ui| {
                ui.label("Keychain");
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
            });

            ui.add_space(7.0);
            ui.horizontal(|ui| {
                ui.label("Password");
                ui.add_sized(
                    [ui.available_width(), 18.0],
                    TextEdit::singleline(&mut app.layouts.start.password).password(true),
                );
            });
            ui.add_space(7.0);
            if let Some(error) = &app.layouts.start.error {
                ui.label(RichText::new(error).color(Color32::RED));
            }
            ui.add_space(10.0);

            let is_ready: bool =
                !app.layouts.start.name.is_empty() && !app.layouts.start.password.is_empty();
            let mut button = ui.add_enabled(is_ready, Button::new("Unlock"));
            button.rect.min.x = 100.0;
            button.rect.max.x = 100.0;

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
    });
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{self, Response, Ui, WidgetText};

const BUTTON_SIZE: [f32; 2] = [180.0, 20.0];

pub struct Button {
    text: WidgetText,
    enabled: bool,
}

impl Button {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<WidgetText>,
    {
        Self {
            text: text.into(),
            enabled: true,
        }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    pub fn render(self, ui: &mut Ui) -> Response {
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.add_sized(BUTTON_SIZE, egui::Button::new(self.text))
        })
        .inner
    }
}

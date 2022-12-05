// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{self, Response, Ui, WidgetText};
use eframe::epaint::Color32;

use crate::gui::theme::color::{DARK_GRAY, WHITE};

const BUTTON_SIZE: [f32; 2] = [200.0, 22.0];

pub struct Button {
    text: WidgetText,
    text_color: Color32,
    background_color: Color32,
    enabled: bool,
}

impl Button {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<WidgetText>,
    {
        Self {
            text: text.into(),
            text_color: WHITE,
            background_color: DARK_GRAY,
            enabled: true,
        }
    }

    #[allow(dead_code)]
    pub fn text_color(self, color: Color32) -> Self {
        Self {
            text_color: color,
            ..self
        }
    }

    pub fn background_color(self, color: Color32) -> Self {
        Self {
            background_color: color,
            ..self
        }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    pub fn render(self, ui: &mut Ui) -> Response {
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.add_sized(
                BUTTON_SIZE,
                egui::Button::new(self.text.color(self.text_color)).fill(self.background_color),
            )
        })
        .inner
    }
}

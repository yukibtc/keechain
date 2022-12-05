// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{self, Response, RichText, Ui};
use eframe::epaint::{Color32, Vec2};

use crate::gui::theme::color::{DARK_GRAY, WHITE};

const BUTTON_SIZE: Vec2 = egui::vec2(210.0, 28.0);

pub struct Button {
    text: String,
    text_color: Color32,
    background_color: Color32,
    enabled: bool,
}

impl Button {
    pub fn new<S>(text: S) -> Self
    where
        S: Into<String>,
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
            let text = RichText::new(self.text).color(self.text_color);
            ui.add_sized(
                BUTTON_SIZE,
                egui::Button::new(text).fill(self.background_color),
            )
        })
        .inner
    }
}

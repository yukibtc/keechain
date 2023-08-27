// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{self, Align, Layout, TextBuffer, TextEdit, Ui, WidgetText};

use crate::GENERIC_FONT_HEIGHT;

const INPUT_FIELD_HEIGHT: f32 = 28.0;

pub struct InputField {
    label: WidgetText,
    placeholder: Option<WidgetText>,
    password: bool,
    rows: u8,
    enabled: bool,
}

impl InputField {
    pub fn new<T>(label: T) -> Self
    where
        T: Into<WidgetText>,
    {
        Self {
            label: label.into(),
            placeholder: None,
            password: false,
            rows: 1,
            enabled: true,
        }
    }

    pub fn placeholder<T>(self, placeholder: T) -> Self
    where
        T: Into<WidgetText>,
    {
        Self {
            placeholder: Some(placeholder.into()),
            ..self
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_password(self) -> Self {
        Self {
            password: true,
            ..self
        }
    }

    pub fn rows(self, rows: u8) -> Self {
        Self { rows, ..self }
    }

    #[allow(dead_code)]
    pub fn enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    pub fn render(self, ui: &mut Ui, text: &mut dyn TextBuffer) {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            let mut widget: TextEdit = if self.rows > 1 {
                TextEdit::multiline(text).desired_rows(self.rows as usize)
            } else {
                TextEdit::singleline(text)
            };

            widget = widget.password(self.password).margin(egui::vec2(
                4.0,
                (INPUT_FIELD_HEIGHT - GENERIC_FONT_HEIGHT) / 2.0,
            ));

            if let Some(placeholder) = self.placeholder {
                widget = widget.hint_text(placeholder);
            }

            ui.label(self.label);
            ui.add_space(0.5);
            ui.add_enabled_ui(self.enabled, |ui| {
                ui.add_sized([ui.available_width(), INPUT_FIELD_HEIGHT], widget);
            });
        });
    }
}

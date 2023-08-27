// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{self, Align, Layout, TextEdit, Ui, WidgetText};

use crate::GENERIC_FONT_HEIGHT;

const FIELD_HEIGHT: f32 = 28.0;

pub struct ReadOnlyField {
    label: WidgetText,
    text: String,
    rows: u8,
}

impl ReadOnlyField {
    pub fn new<T, S>(label: T, text: S) -> Self
    where
        T: Into<WidgetText>,
        S: Into<String>,
    {
        Self {
            label: label.into(),
            text: text.into(),
            rows: 1,
        }
    }

    pub fn rows(self, rows: u8) -> Self {
        Self { rows, ..self }
    }

    pub fn render(self, ui: &mut Ui) {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            let mut text = self.text.clone();
            let mut widget: TextEdit = if self.rows > 1 {
                TextEdit::multiline(&mut text).desired_rows(self.rows as usize)
            } else {
                TextEdit::singleline(&mut text)
            };

            widget = widget.margin(egui::vec2(4.0, (FIELD_HEIGHT - GENERIC_FONT_HEIGHT) / 2.0));

            ui.label(self.label);
            ui.add_space(0.5);
            ui.add_sized([ui.available_width(), FIELD_HEIGHT], widget);
        });
    }
}

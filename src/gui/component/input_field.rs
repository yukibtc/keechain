// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Layout, Ui, Widget, WidgetText};

pub struct InputField {
    label: WidgetText,
}

impl InputField {
    pub fn new<T>(label: T) -> Self
    where
        T: Into<WidgetText>,
    {
        Self {
            label: label.into(),
        }
    }

    pub fn render(self, ui: &mut Ui, widget: impl Widget) {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.label(self.label);
            ui.add_space(0.7);
            ui.add_sized([ui.available_width(), 22.0], widget);
        });
    }
}

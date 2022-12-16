// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Layout, ScrollArea, Ui};

use super::Version;

pub struct View;

impl View {
    pub fn show<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.set_max_width(ui.available_width() - 20.0);
                add_contents(ui);
            });
            ui.add_space(20.0);
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                Version::new().render(ui)
            });
        });
    }
}

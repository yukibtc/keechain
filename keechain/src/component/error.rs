// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{RichText, Ui};

use crate::theme::color::RED;

pub struct Error {
    text: String,
}

impl Error {
    pub fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self { text: text.into() }
    }

    pub fn render(self, ui: &mut Ui) {
        ui.label(RichText::new(self.text).color(RED));
    }
}

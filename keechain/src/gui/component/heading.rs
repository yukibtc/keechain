// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Response, RichText, Ui};

pub struct Heading {
    text: RichText,
}

impl Heading {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<RichText>,
    {
        Self { text: text.into() }
    }

    pub fn render(self, ui: &mut Ui) -> Response {
        ui.heading(self.text)
    }
}

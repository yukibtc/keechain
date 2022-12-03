// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Response, RichText, Ui};

pub struct Version {
    text: RichText,
}

impl Version {
    pub fn new() -> Self {
        Self {
            text: RichText::new(format!(
                "{} v{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .small(),
        }
    }

    pub fn render(self, ui: &mut Ui) -> Response {
        ui.label(self.text)
    }
}

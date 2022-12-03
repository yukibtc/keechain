// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Response, RichText, Ui};

pub struct Version {
    text: RichText,
}

impl Version {
    pub fn new() -> Self {
        let mut text = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        if cfg!(debug_assertions) {
            text.push_str(" (debug)");
        }

        Self {
            text: RichText::new(text).small(),
        }
    }

    pub fn render(self, ui: &mut Ui) -> Response {
        ui.label(self.text)
    }
}

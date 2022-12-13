// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Response, RichText, Ui};

const HEADING_SIZE: f32 = 28.0;

pub struct Heading {
    text: RichText,
    size: f32,
}

impl Heading {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<RichText>,
    {
        Self {
            text: text.into(),
            size: HEADING_SIZE,
        }
    }

    #[allow(dead_code)]
    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }

    pub fn render(self, ui: &mut Ui) -> Response {
        ui.label(self.text.heading().size(self.size))
    }
}

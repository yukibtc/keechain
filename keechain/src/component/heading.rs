// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{RichText, Ui};

const HEADING_SIZE: f32 = 28.0;
const MARGIN_BOTTOM: f32 = 10.0;

pub struct Heading {
    text: RichText,
    size: f32,
    margin_bottom: f32,
}

impl Heading {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<RichText>,
    {
        Self {
            text: text.into(),
            size: HEADING_SIZE,
            margin_bottom: MARGIN_BOTTOM,
        }
    }

    #[allow(dead_code)]
    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }

    #[allow(dead_code)]
    pub fn margin_bottom(self, margin_bottom: f32) -> Self {
        Self {
            margin_bottom,
            ..self
        }
    }

    pub fn render(self, ui: &mut Ui) {
        ui.label(self.text.heading().size(self.size));
        ui.add_space(self.margin_bottom);
    }
}

// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use iced::widget::rule::FillMode;
use iced::widget::{rule, Rule};
use iced::{theme, BorderRadius, Renderer, Theme};

use crate::theme::color::GREY;

pub fn horizontal() -> Rule<Renderer> {
    Rule::horizontal(1)
}

pub fn horizontal_bold() -> Rule<Renderer> {
    Rule::horizontal(1).style(theme::Rule::Custom(Box::new(BoldRuleStyle)))
}

pub fn vertical() -> Rule<Renderer> {
    Rule::vertical(1)
}

pub struct BoldRuleStyle;

impl rule::StyleSheet for BoldRuleStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> rule::Appearance {
        rule::Appearance {
            width: 3,
            color: GREY,
            fill_mode: FillMode::Full,
            radius: BorderRadius::default(),
        }
    }
}

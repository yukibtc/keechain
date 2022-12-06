// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bitcoin::Network;
use eframe::egui::{self, CentralPanel, Context};
use eframe::epaint::FontFamily::Proportional;
use eframe::epaint::{FontId, Vec2};
use eframe::{App, Frame, NativeOptions, Theme};
use egui::TextStyle::{Body, Button, Heading, Monospace, Small};

mod component;
mod layout;
mod theme;

use self::layout::sign::SignLayoutData;
use self::layout::{RestoreLayoutData, StartLayoutData};
use crate::core::types::Seed;

const MIN_WINDOWS_SIZE: Vec2 = egui::vec2(350.0, 530.0);
const GENERIC_FONT_HEIGHT: f32 = 18.0;

pub fn launch(network: Network) -> Result<()> {
    let options = NativeOptions {
        fullscreen: false,
        resizable: true,
        always_on_top: true,
        default_theme: Theme::Dark,
        follow_system_theme: false,
        initial_window_size: Some(MIN_WINDOWS_SIZE),
        min_window_size: Some(MIN_WINDOWS_SIZE),
        drag_and_drop_support: false,
        ..Default::default()
    };
    let app = AppData::new(&network);
    let app_name = format!(
        "KeeChain{}",
        if network.ne(&Network::Bitcoin) {
            format!(" [{}]", network)
        } else {
            String::new()
        }
    );
    eframe::run_native(&app_name, options, Box::new(|_cc| Box::new(app)));
    Ok(())
}

#[derive(Clone)]
pub enum Menu {
    Main,
    Advanced,
    Setting,
    Danger,
}

#[derive(Clone)]
pub enum AppStage {
    Start,
    NewKeychain,
    RestoreKeychain,
    Menu(Menu),
    Sign,
}

impl Default for AppStage {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Clone, Default)]
pub struct AppLayoutData {
    start: StartLayoutData,
    restore: RestoreLayoutData,
    sign: SignLayoutData,
}

#[derive(Clone)]
pub struct AppData {
    network: Network,
    stage: AppStage,
    seed: Option<Seed>,
    layouts: AppLayoutData,
}

impl AppData {
    pub fn new(network: &Network) -> Self {
        Self {
            network: *network,
            stage: AppStage::default(),
            seed: None,
            layouts: AppLayoutData::default(),
        }
    }

    fn set_stage(&mut self, stage: AppStage) {
        self.stage = stage;
    }

    fn set_seed(&mut self, seed: Option<Seed>) {
        self.seed = seed;
    }
}

impl App for AppData {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(28.0, Proportional)),
            (Body, FontId::new(GENERIC_FONT_HEIGHT, Proportional)),
            (Monospace, FontId::new(GENERIC_FONT_HEIGHT, Proportional)),
            (Button, FontId::new(GENERIC_FONT_HEIGHT, Proportional)),
            (Small, FontId::new(14.0, Proportional)),
        ]
        .into();
        ctx.set_style(style);

        CentralPanel::default().show(ctx, |ui| match &self.stage {
            AppStage::Start => layout::start::update_layout(self, ui),
            AppStage::NewKeychain => todo!(),
            AppStage::RestoreKeychain => layout::restore::update_layout(self, ui),
            AppStage::Menu(menu) => layout::menu::update_layout(self, menu.clone(), ui, frame),
            AppStage::Sign => layout::sign::update_layout(self, ui),
        });
    }
}

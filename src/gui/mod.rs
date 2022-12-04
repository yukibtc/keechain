// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bitcoin::Network;
use eframe::egui::{self, CentralPanel, Context};
use eframe::epaint::FontFamily::Proportional;
use eframe::epaint::{FontId, Vec2};
use eframe::{App, Frame, NativeOptions};
use egui::TextStyle::*;

mod component;
mod layout;

use self::component::Button;
use self::layout::StartLayoutData;
use crate::types::Seed;

const MIN_WINDOWS_SIZE: Vec2 = egui::vec2(320.0, 500.0);

pub fn launch(network: Network) -> Result<()> {
    let options = NativeOptions {
        fullscreen: false,
        resizable: false,
        initial_window_size: Some(MIN_WINDOWS_SIZE),
        min_window_size: Some(MIN_WINDOWS_SIZE),
        ..Default::default()
    };
    let app = AppData::new(&network);
    eframe::run_native("KeeChain", options, Box::new(|_cc| Box::new(app)));
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
pub enum Command {
    Sign,
}

#[derive(Clone)]
pub enum AppStage {
    Start,
    NewKeychain,
    RestoreKeychain,
    Menu(Menu),
    Command(Command),
}

impl Default for AppStage {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Clone, Default)]
pub struct AppLayoutData {
    start: StartLayoutData,
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
            (Body, FontId::new(18.0, Proportional)),
            (Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(18.0, Proportional)),
            (Small, FontId::new(14.0, Proportional)),
        ]
        .into();
        ctx.set_style(style);

        CentralPanel::default().show(ctx, |ui| match &self.stage {
            AppStage::Start => layout::start::update_layout(self, ctx),
            AppStage::NewKeychain => todo!(),
            AppStage::RestoreKeychain => todo!(),
            AppStage::Menu(menu) => layout::menu::update_layout(self, menu.clone(), ctx, frame),
            AppStage::Command(_command) => {}
        });
    }
}
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bitcoin::Network;
use eframe::egui::{self, CentralPanel, Context};
use eframe::epaint::FontFamily::Proportional;
use eframe::epaint::{FontId, Vec2};
use eframe::{App, Frame, NativeOptions};
use egui::TextStyle::*;

mod layout;

use self::layout::open::OpenLayout;
use crate::types::Seed;

const MIN_WINDOWS_SIZE: Vec2 = egui::vec2(320.0, 500.0);

pub fn launch(network: Network) -> Result<()> {
    let options = NativeOptions {
        fullscreen: false,
        initial_window_size: Some(MIN_WINDOWS_SIZE),
        min_window_size: Some(MIN_WINDOWS_SIZE),
        ..Default::default()
    };

    let app = AppData::new(&network);

    eframe::run_native("KeeChain", options, Box::new(|_cc| Box::new(app)));
    Ok(())
}

#[derive(Clone)]
enum Menu {
    Main,
    Advanced,
    Setting,
    Danger,
}

#[derive(Clone)]
enum Command {
    Sign,
}

#[derive(Clone)]
enum AppStage {
    Open,
    Menu(Menu),
    Command(Command),
}

impl Default for AppStage {
    fn default() -> Self {
        Self::Open
    }
}

#[derive(Clone)]
pub struct AppData {
    network: Network,
    stage: AppStage,
    seed: Option<Seed>,
    open_layout: OpenLayout,
}

impl AppData {
    pub fn new(network: &Network) -> Self {
        Self {
            network: *network,
            stage: AppStage::default(),
            seed: None,
            open_layout: OpenLayout::default(),
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
            AppStage::Open => layout::open::update_layout(self, ctx),
            AppStage::Menu(menu) => {
                ui.heading("Menu");
                if ui.button("Lock").clicked() {
                    self.stage = AppStage::Open;
                }
                if ui.button("Exit").clicked() {
                    frame.close();
                }
            }
            AppStage::Command(command) => {}
        });
    }
}

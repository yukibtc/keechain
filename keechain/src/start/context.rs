// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use keechain_core::bitcoin::Network;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Open,
    New,
    Restore,
}

impl Default for Stage {
    fn default() -> Self {
        Self::Open
    }
}

pub struct Context {
    pub stage: Stage,
    pub network: Network,
}

impl Context {
    pub fn new(stage: Stage, network: Network) -> Self {
        Self { stage, network }
    }

    pub fn set_stage(&mut self, stage: Stage) {
        self.stage = stage;
    }
}

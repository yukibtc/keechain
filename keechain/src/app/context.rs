// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use keechain_core::{bitcoin::Network, keychain::KeeChain};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Home,
    Setting,
}

impl Default for Stage {
    fn default() -> Self {
        Self::Home
    }
}

pub struct Context {
    pub stage: Stage,
    pub network: Network,
    pub keechain: KeeChain,
}

impl Context {
    pub fn new(stage: Stage, network: Network, keechain: KeeChain) -> Self {
        Self {
            stage,
            network,
            keechain,
        }
    }

    pub fn set_stage(&mut self, stage: Stage) {
        self.stage = stage;
    }
}

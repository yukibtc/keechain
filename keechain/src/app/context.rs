// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use keechain_core::KeeChain;

use crate::component::Identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage {
    Home,
    Sign,
}

impl Default for Stage {
    fn default() -> Self {
        Self::Home
    }
}

pub struct Context {
    pub stage: Stage,
    pub keechain: KeeChain,
}

impl Context {
    pub fn new(stage: Stage, keechain: KeeChain) -> Self {
        Self {
            stage: stage.clone(),
            keechain,
        }
    }

    pub fn set_stage(&mut self, stage: Stage) {
        self.stage = stage;
    }

    pub fn identity(&self) -> Identity {
        Identity::new(self.keechain.identity(), self.keechain.has_passphrase())
    }
}

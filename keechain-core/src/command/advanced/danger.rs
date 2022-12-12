// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::Network;

use crate::error::Result;
use crate::types::{Secrets, Seed};

pub fn view_secrets(seed: Seed, network: Network) -> Result<Secrets> {
    Secrets::new(seed, network)
}

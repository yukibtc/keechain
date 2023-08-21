// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

use bdk::miniscript::descriptor::{Descriptor, DescriptorPublicKey};
use bitcoin::Network;
use serde::Serialize;
use serde_json::json;

use super::{Descriptors, Seed};

#[derive(Debug)]
pub enum Error {
    Descriptor(super::descriptors::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Descriptor(e) => write!(f, "Descriptor: {e}"),
        }
    }
}

impl From<super::descriptors::Error> for Error {
    fn from(e: super::descriptors::Error) -> Self {
        Self::Descriptor(e)
    }
}

#[derive(Debug, Serialize)]
pub struct BitcoinCoreDescriptor {
    timestamp: String,
    active: bool,
    desc: Descriptor<DescriptorPublicKey>,
    internal: bool,
}

impl BitcoinCoreDescriptor {
    pub fn new(desc: Descriptor<DescriptorPublicKey>, internal: bool) -> Self {
        Self {
            timestamp: String::from("now"),
            active: true,
            desc,
            internal,
        }
    }
}

#[derive(Debug)]
pub struct BitcoinCore(Vec<BitcoinCoreDescriptor>);

impl BitcoinCore {
    pub fn new(seed: Seed, network: Network, account: Option<u32>) -> Result<Self, Error> {
        let descriptors: Descriptors = Descriptors::new(seed, network, account)?;
        let mut bitcoin_core_descriptors: Vec<BitcoinCoreDescriptor> = Vec::new();

        for desc in descriptors.external().into_iter() {
            bitcoin_core_descriptors.push(BitcoinCoreDescriptor::new(desc, false));
        }

        for desc in descriptors.internal().into_iter() {
            bitcoin_core_descriptors.push(BitcoinCoreDescriptor::new(desc, true));
        }

        Ok(Self(bitcoin_core_descriptors))
    }
}

impl ToString for BitcoinCore {
    fn to_string(&self) -> String {
        format!("\nimportdescriptors '{}'\n", json!(self.0))
    }
}

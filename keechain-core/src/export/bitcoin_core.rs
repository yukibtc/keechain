// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

use bdk::bitcoin::secp256k1::{Secp256k1, Signing};
use bdk::bitcoin::Network;
use bdk::miniscript::descriptor::{Descriptor, DescriptorPublicKey};
use serde::Serialize;
use serde_json::json;

use crate::{descriptors, Descriptors, Seed};

#[derive(Debug)]
pub enum Error {
    Descriptor(descriptors::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Descriptor(e) => write!(f, "Descriptor: {e}"),
        }
    }
}

impl From<descriptors::Error> for Error {
    fn from(e: descriptors::Error) -> Self {
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
    pub fn new<C>(
        seed: &Seed,
        network: Network,
        account: Option<u32>,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        C: Signing,
    {
        let descriptors: Descriptors = Descriptors::new(seed, network, account, secp)?;
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

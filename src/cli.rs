// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;

use bitcoin::Network;
use clap::{Parser, Subcommand};

use crate::command::export::ElectrumExportSupportedScripts;
use crate::types::{Index, WordCount};

#[derive(Debug, Parser)]
#[command(name = "keechain")]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Network
    #[clap(short, long, default_value_t = Network::Bitcoin)]
    pub network: Network,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Restore BIP39 Seed Phrase
    Restore {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
    /// Get fingerprint
    Identity {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
    /// Export
    #[command(arg_required_else_help = true)]
    Export {
        /// Type
        #[command(subcommand)]
        export_type: ExportTypes,
    },
    /// Derive BIP39 Seed Phrase (BIP85)
    #[command(arg_required_else_help = true)]
    Derive {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// Word count
        #[arg(required = true, value_enum)]
        word_count: WordCount,
        /// Index (must be between 0 and 2^31 - 1)
        #[arg(required = true)]
        index: Index,
    },
    /// Sign PSBT
    #[command(arg_required_else_help = true)]
    Sign {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// PSBT file
        #[arg(required = true)]
        file: PathBuf,
    },
    /// Danger
    Danger {
        #[command(subcommand)]
        command: DangerCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum DangerCommands {
    /// View mnemonic and passphrase
    ViewSeed {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
    /// Delete keychain
    Wipe {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ExportTypes {
    /// Export descriptors
    Descriptors {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// Account number
        #[arg(default_value_t = 0)]
        account: u32,
    },
    /// Export Bitcoin Core descriptors
    BitcoinCore {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// Account number
        #[arg(default_value_t = 0)]
        account: u32,
    },
    /// Export Electrum file
    Electrum {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// Script
        #[arg(default_value_t = ElectrumExportSupportedScripts::NativeSegwit)]
        script: ElectrumExportSupportedScripts,
        /// Account number
        #[arg(default_value_t = 0)]
        account: u32,
    },
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;

use bitcoin::Network;
use clap::{Parser, Subcommand};

use crate::types::{ExportTypes, Index, WordCount};

#[derive(Debug, Parser)]
#[command(name = "keechain")]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Network
    #[clap(short, long, default_value_t = Network::Bitcoin)]
    pub network: Network,
    /// Keychain name
    #[arg(required = true)]
    pub name: String,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Restore BIP39 Seed Phrase
    Restore,
    /// Export descriptors
    #[command(arg_required_else_help = true)]
    Export {
        /// Type
        #[arg(required = true, name = "TYPE")]
        export_type: ExportTypes,
        /// Account number
        #[arg(default_value_t = 0)]
        account: u32,
    },
    /// Derive BIP39 Seed Phrase with Deterministic Entropy (BIP85)
    #[command(arg_required_else_help = true)]
    Derive {
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
    ViewSeed,
    /// Delete keychain
    Wipe,
}

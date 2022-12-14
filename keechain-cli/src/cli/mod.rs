// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use keechain_core::bitcoin::Network;
use keechain_core::types::{ElectrumExportSupportedScripts, Index, WordCount};

pub mod io;

#[derive(Debug, Parser)]
#[command(name = "keechain")]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Network
    #[clap(short, long, default_value_t = Network::Bitcoin)]
    pub network: Network,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate mnemonic (BIP39)
    #[command(arg_required_else_help = true)]
    Generate {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// Word count
        #[arg(value_enum, default_value_t = WordCount::W24)]
        word_count: WordCount,
        /// Add entropy from dice roll
        #[arg(long, default_value_t = false)]
        dice_roll: bool,
    },
    /// Restore mnemonic (BIP39)
    #[command(arg_required_else_help = true)]
    Restore {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
    /// List keychains
    List,
    /// View master fingerprint
    #[command(arg_required_else_help = true)]
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
    /// Decode PSBT
    #[command(arg_required_else_help = true)]
    Decode {
        /// PSBT file
        #[arg(required = true)]
        file: PathBuf,
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
    /// Advanced
    Advanced {
        #[command(subcommand)]
        command: AdvancedCommand,
    },
    /// Setting
    Setting {
        #[command(subcommand)]
        command: SettingCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum AdvancedCommand {
    /// Deterministic entropy (BIP85)
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
    Vanity {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// Prefixes
        #[arg(required = true)]
        prefixes: Vec<String>,
    },
    /// Danger
    Danger {
        #[command(subcommand)]
        command: DangerCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum DangerCommand {
    /// View secrets
    #[command(arg_required_else_help = true)]
    ViewSecrets {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
    /// Delete keychain
    #[command(arg_required_else_help = true)]
    Wipe {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SettingCommand {
    /// Rename keychain
    #[command(arg_required_else_help = true)]
    Rename {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// New keychain name
        #[arg(required = true)]
        new_name: String,
    },
    /// Change keychain password
    #[command(arg_required_else_help = true)]
    ChangePassword {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ExportTypes {
    /// Export descriptors
    #[command(arg_required_else_help = true)]
    Descriptors {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// Account number
        #[arg(default_value_t = 0)]
        account: u32,
    },
    /// Export Bitcoin Core descriptors
    #[command(arg_required_else_help = true)]
    BitcoinCore {
        /// Keychain name
        #[arg(required = true)]
        name: String,
        /// Account number
        #[arg(default_value_t = 0)]
        account: u32,
    },
    /// Export Electrum file
    #[command(arg_required_else_help = true)]
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

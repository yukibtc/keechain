// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use keechain_core::bdk::miniscript::Descriptor;
use keechain_core::types::Index;

pub mod io;

use crate::types::{CliElectrumSupportedScripts, CliNetwork, CliWordCount};

#[derive(Debug, Parser)]
#[command(name = "keechain")]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Network
    #[clap(short, long, value_enum, default_value_t = CliNetwork::Bitcoin)]
    pub network: CliNetwork,
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
        #[arg(value_enum, default_value_t = CliWordCount::W24)]
        word_count: CliWordCount,
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
        /// Print base64
        #[clap(long)]
        base64: bool,
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
        /// Descriptor (optional)
        descriptor: Option<Descriptor<String>>,
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
        word_count: CliWordCount,
        /// Index (must be between 0 and 2^31 - 1)
        #[arg(required = true)]
        index: Index,
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
        #[arg(value_enum, default_value_t = CliElectrumSupportedScripts::NativeSegwit)]
        script: CliElectrumSupportedScripts,
        /// Account number
        #[arg(default_value_t = 0)]
        account: u32,
    },
    /// Export Wasabi file
    #[command(arg_required_else_help = true)]
    Wasabi {
        /// Keychain name
        #[arg(required = true)]
        name: String,
    },
}

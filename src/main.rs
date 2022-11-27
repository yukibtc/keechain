// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bitcoin::Network;
use clap::Parser;
use types::ExportTypes;

mod cli;
mod command;
mod types;
mod util;

use self::cli::{Cli, Commands, DangerCommands};
use self::util::io;

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    let network: Network = args.network;
    let name: String = args.name;

    let password: String = rpassword::prompt_password("Password: ")?;

    match args.command {
        Commands::Restore => {
            let seed: String = io::get_input("Seed: ")?;
            let passphrase: Option<String> =
                if let Ok(result) = io::ask("Do you want to use a passphrase?") {
                    if result {
                        Some(io::get_input("Passphrase: ")?)
                    } else {
                        None
                    }
                } else {
                    None
                };

            command::restore(name, password, seed, passphrase)
        }
        Commands::Export {
            export_type,
            account,
        } => match export_type {
            ExportTypes::Descriptors => {
                command::get_public_keys(name, password, network, Some(account))
            }
        },
        Commands::Derive { word_count, index } => {
            command::derive(name, password, network, word_count, index)
        }
        Commands::Sign { file } => command::sign(name, password, network, file),
        Commands::Danger { command } => match command {
            DangerCommands::ViewSeed => command::view_seed(name, password),
            DangerCommands::Wipe => {
                if io::ask("Are you really sure? This action is permanent!")? && io::ask("Again, are you really sure? THIS ACTION IS PERMANENT AND YOU MAY LOSE ALL YOUR FUNDS!")? {
                    command::wipe(name, password)?;
                } else {
                    println!("Aborted.");
                }
                Ok(())
            }
        },
    }
}

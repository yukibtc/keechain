// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bitcoin::Network;
use clap::Parser;

mod cli;
mod command;
mod types;
mod util;

use self::cli::{Cli, Commands, DangerCommands, ExportTypes};
use self::util::bip::bip32;
use self::util::io;

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    let network: Network = args.network;

    match args.command {
        Commands::Generate { name, word_count } => {
            let mnemonic = command::generate(
                name,
                || io::get_password("Password: "),
                || {
                    if let Ok(result) = io::ask("Do you want to use a passphrase?") {
                        if result {
                            Ok(Some(io::get_input("Passphrase: ")?))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                },
                word_count,
            )?;

            println!("\n!!! WRITE DOWN YOUT SEED PHRASE !!!");
            println!("\n################################################################\n");
            println!("{}", mnemonic);
            println!("\n################################################################\n");

            Ok(())
        }
        Commands::Restore { name } => command::restore(
            name,
            || io::get_password("Password: "),
            || io::get_input("Seed: "),
            || {
                if let Ok(result) = io::ask("Do you want to use a passphrase?") {
                    if result {
                        Ok(Some(io::get_input("Passphrase: ")?))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            },
        ),
        Commands::Identity { name } => {
            command::identity(name, || io::get_password("Password: "), network)
        }
        Commands::Export { export_type } => match export_type {
            ExportTypes::Descriptors { name, account } => {
                let descriptors = command::export::descriptors(
                    name,
                    || io::get_password("Password: "),
                    network,
                    Some(account),
                )?;
                println!("{:#?}", descriptors);
                Ok(())
            }
            ExportTypes::BitcoinCore { name, account } => command::export::bitcoin_core(
                name,
                || io::get_password("Password: "),
                network,
                Some(account),
            ),
            ExportTypes::Electrum {
                name,
                script,
                account,
            } => command::export::electrum(
                name,
                || io::get_password("Password: "),
                network,
                bip32::account_extended_path(script.as_u32(), network, Some(account))?,
            ),
        },
        Commands::Derive {
            name,
            word_count,
            index,
        } => command::derive(
            name,
            || io::get_password("Password: "),
            network,
            word_count,
            index,
        ),
        Commands::Decode { file } => {
            let psbt = command::decode(file)?;
            println!("{:#?}", psbt);
            Ok(())
        }
        Commands::Sign { name, file } => {
            command::sign(name, || io::get_password("Password: "), network, file)
        }
        Commands::Danger { command } => match command {
            DangerCommands::ViewSeed { name } => {
                command::danger::view_seed(name, || io::get_password("Password: "), network)
            }
            DangerCommands::Wipe { name } => {
                if io::ask("Are you really sure? This action is permanent!")? && io::ask("Again, are you really sure? THIS ACTION IS PERMANENT AND YOU MAY LOSE ALL YOUR FUNDS!")? {
                    command::danger::wipe(name, || io::get_password("Password: "))?;
                } else {
                    println!("Aborted.");
                }
                Ok(())
            }
        },
    }
}

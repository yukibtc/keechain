// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use anyhow::Result;
use bdk::keys::bip39::Mnemonic;
use bitcoin::Network;
use clap::Parser;
use console::Term;

mod cli;
mod command;
mod types;
mod util;

use self::cli::{AdvancedCommands, Cli, Commands, DangerCommands, ExportTypes, SettingCommands};
use self::util::bip::bip32;
use self::util::io;

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    let network: Network = args.network;

    match args.command {
        Commands::Generate {
            name,
            word_count,
            dice_roll,
        } => {
            let mnemonic = command::generate(
                name,
                io::get_password_with_confirmation,
                || {
                    if io::ask("Do you want to use a passphrase?")? {
                        Ok(Some(io::get_input("Passphrase")?))
                    } else {
                        Ok(None)
                    }
                },
                word_count,
                || {
                    if dice_roll {
                        let term = Term::stdout();
                        let mut rolls: Vec<u8> = Vec::new();
                        io::select_dice_roll(term, &mut rolls)?;
                        Ok(Some(rolls))
                    } else {
                        Ok(None)
                    }
                },
            )?;

            println!("\n!!! WRITE DOWN YOUT SEED PHRASE !!!");
            println!("\n################################################################\n");
            println!("{}", mnemonic);
            println!("\n################################################################\n");

            Ok(())
        }
        Commands::Restore { name } => command::restore(
            name,
            io::get_password_with_confirmation,
            || Ok(Mnemonic::from_str(&io::get_input("Seed")?)?),
            || {
                if io::ask("Do you want to use a passphrase?")? {
                    Ok(Some(io::get_input("Passphrase")?))
                } else {
                    Ok(None)
                }
            },
        ),
        Commands::List => {
            let names = util::dir::get_keychains_list()?;
            for (index, name) in names.iter().enumerate() {
                println!("{}. {}", index + 1, name);
            }
            Ok(())
        }
        Commands::Identity { name } => command::identity(name, io::get_password, network),
        Commands::Export { export_type } => match export_type {
            ExportTypes::Descriptors { name, account } => {
                let descriptors =
                    command::export::descriptors(name, io::get_password, network, Some(account))?;
                println!("{:#?}", descriptors);
                Ok(())
            }
            ExportTypes::BitcoinCore { name, account } => {
                command::export::bitcoin_core(name, io::get_password, network, Some(account))
            }
            ExportTypes::Electrum {
                name,
                script,
                account,
            } => command::export::electrum(
                name,
                io::get_password,
                network,
                bip32::account_extended_path(script.as_u32(), network, Some(account))?,
            ),
        },
        Commands::Decode { file } => {
            let psbt = command::decode(file)?;
            println!("{:#?}", psbt);
            Ok(())
        }
        Commands::Sign { name, file } => command::sign(name, io::get_password, network, file),
        Commands::Advanced { command } => match command {
            AdvancedCommands::Derive {
                name,
                word_count,
                index,
            } => command::advanced::derive(name, io::get_password, network, word_count, index),
            AdvancedCommands::Danger { command } => match command {
                DangerCommands::ViewSeed { name } => {
                    command::advanced::danger::view_seed(name, io::get_password, network)
                }
                DangerCommands::Wipe { name } => {
                    if io::ask("Are you really sure? This action is permanent!")? && io::ask("Again, are you really sure? THIS ACTION IS PERMANENT AND YOU MAY LOSE ALL YOUR FUNDS!")? {
                        command::advanced::danger::wipe(name, io::get_password)?;
                    } else {
                        println!("Aborted.");
                    }
                    Ok(())
                }
            },
        },
        Commands::Setting { command } => match command {
            SettingCommands::Rename { name, new_name } => command::setting::rename(name, new_name),
            SettingCommands::ChangePassword { name } => command::setting::change_password(
                name,
                io::get_password,
                io::get_password_with_confirmation,
            ),
        },
    }
}

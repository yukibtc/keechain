// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use anyhow::Result;
use bdk::keys::bip39::Mnemonic;
use bitcoin::Network;
use clap::Parser;
use console::Term;

mod cli;
mod core;
#[cfg(feature = "gui")]
mod gui;

use self::cli::io;
use self::cli::{AdvancedCommand, Cli, Command, DangerCommand, ExportTypes, SettingCommand};
use self::core::command;
use self::core::util::bip::bip32;
use self::core::util::dir;

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    let network: Network = args.network;

    match args.command {
        #[cfg(feature = "gui")]
        Command::Launch => gui::launch(network),
        Command::Generate {
            name,
            word_count,
            dice_roll,
        } => {
            let seed = command::generate(
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
            println!("{}", seed.mnemonic());
            println!("\n################################################################\n");

            Ok(())
        }
        Command::Restore { name } => {
            command::restore(
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
            )?;
            Ok(())
        }
        Command::List => {
            let names = dir::get_keychains_list()?;
            for (index, name) in names.iter().enumerate() {
                println!("{}. {}", index + 1, name);
            }
            Ok(())
        }
        Command::Identity { name } => {
            let fingerprint = command::identity(name, io::get_password, network)?;
            println!("Fingerprint: {}", fingerprint);
            Ok(())
        }
        Command::Export { export_type } => match export_type {
            ExportTypes::Descriptors { name, account } => {
                let descriptors =
                    command::export::descriptors(name, io::get_password, network, Some(account))?;
                println!("{:#?}", descriptors);
                Ok(())
            }
            ExportTypes::BitcoinCore { name, account } => {
                let descriptors =
                    command::export::bitcoin_core(name, io::get_password, network, Some(account))?;
                println!("{}", descriptors);
                Ok(())
            }
            ExportTypes::Electrum {
                name,
                script,
                account,
            } => {
                let path = command::export::electrum(
                    name,
                    io::get_password,
                    network,
                    bip32::account_extended_path(script.as_u32(), network, Some(account))?,
                )?;
                println!("Electrum file exported: {}", path.display());
                Ok(())
            }
        },
        Command::Decode { file } => command::decode(file, network)?.print(),
        Command::Sign { name, file } => {
            if command::sign(name, io::get_password, network, file)? {
                println!("Signed.")
            } else {
                println!("PSBT signing not finalized");
            }
            Ok(())
        }
        Command::Advanced { command } => match command {
            AdvancedCommand::Derive {
                name,
                word_count,
                index,
            } => {
                let mnemonic =
                    command::advanced::derive(name, io::get_password, network, word_count, index)?;
                println!("Mnemonic: {}", mnemonic);
                Ok(())
            }
            AdvancedCommand::Danger { command } => match command {
                DangerCommand::ViewSecrets { name } => {
                    let secrets =
                        command::advanced::danger::view_secrets(name, io::get_password, network)?;
                    secrets.print();
                    Ok(())
                }
                DangerCommand::Wipe { name } => {
                    if io::ask("Are you really sure? This action is permanent!")? && io::ask("Again, are you really sure? THIS ACTION IS PERMANENT AND YOU MAY LOSE ALL YOUR FUNDS!")? {
                        command::advanced::danger::wipe(name, io::get_password)?;
                    } else {
                        println!("Aborted.");
                    }
                    Ok(())
                }
            },
        },
        Command::Setting { command } => match command {
            SettingCommand::Rename { name, new_name } => command::setting::rename(name, new_name),
            SettingCommand::ChangePassword { name } => command::setting::change_password(
                name,
                io::get_password,
                io::get_password_with_confirmation,
            ),
        },
    }
}

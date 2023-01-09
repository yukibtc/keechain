// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#![doc = include_str!("../../README.md")]

use std::str::FromStr;

use clap::Parser;
use console::Term;
use keechain_core::bdk::keys::bip39::Mnemonic;
use keechain_core::bitcoin::Network;
use keechain_core::command;
use keechain_core::error::Result;
use keechain_core::keychain::KeeChain;
use keechain_core::util::dir;

mod cli;

use self::cli::io;
use self::cli::{AdvancedCommand, Cli, Command, DangerCommand, ExportTypes, SettingCommand};

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    let network: Network = args.network;

    match args.command {
        Command::Generate {
            name,
            word_count,
            dice_roll,
        } => {
            let keechain =
                KeeChain::generate(name, io::get_password_with_confirmation, word_count, || {
                    if dice_roll {
                        let term = Term::stdout();
                        let mut rolls: Vec<u8> = Vec::new();
                        io::select_dice_roll(term, &mut rolls)?;
                        Ok(Some(rolls))
                    } else {
                        Ok(None)
                    }
                })?;

            println!("\n!!! WRITE DOWN YOUT SEED PHRASE !!!");
            println!("\n################################################################\n");
            println!("{}", keechain.keychain.seed.mnemonic());
            println!("\n################################################################\n");

            Ok(())
        }
        Command::Restore { name } => {
            KeeChain::restore(name, io::get_password_with_confirmation, || {
                Ok(Mnemonic::from_str(&io::get_input("Seed")?)?)
            })?;
            Ok(())
        }
        Command::List => {
            let names = dir::get_keychains_list()?;
            for (index, name) in names.iter().enumerate() {
                println!("{}. {}", index + 1, name);
            }
            Ok(())
        }
        Command::Identity { name: _ } => {
            todo!();
        }
        Command::Export { export_type } => match export_type {
            ExportTypes::Descriptors { name, account } => {
                let keechain = KeeChain::open(name, io::get_password)?;
                let descriptors =
                    command::export::descriptors(keechain.keychain.seed(), network, Some(account))?;
                println!("{:#?}", descriptors);
                Ok(())
            }
            ExportTypes::BitcoinCore { name, account } => {
                let keechain = KeeChain::open(name, io::get_password)?;
                let descriptors = command::export::bitcoin_core(
                    keechain.keychain.seed(),
                    network,
                    Some(account),
                )?;
                println!("{}", descriptors);
                Ok(())
            }
            ExportTypes::Electrum {
                name,
                script,
                account,
            } => {
                let keechain = KeeChain::open(name, io::get_password)?;
                let path = command::export::electrum(
                    keechain.keychain.seed(),
                    network,
                    script,
                    Some(account),
                )?;
                println!("Electrum file exported: {}", path.display());
                Ok(())
            }
        },
        Command::Decode { file } => command::psbt::decode_file(file, network)?.print(),
        Command::Sign { name, file } => {
            let keechain = KeeChain::open(name, io::get_password)?;
            if command::psbt::sign_file_from_seed(&keechain.keychain.seed(), network, file)? {
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
                let keechain = KeeChain::open(name, io::get_password)?;
                let mnemonic: Mnemonic = keechain
                    .keychain
                    .deterministic_entropy(network, word_count, index)?;
                println!("Mnemonic: {}", mnemonic);
                Ok(())
            }
            AdvancedCommand::Vanity { name, prefixes } => {
                let keechain = KeeChain::open(name, io::get_password)?;
                let (path, address) = keechain_core::command::vanity::search_address(
                    keechain.keychain.seed(),
                    prefixes,
                    network,
                )?;
                println!("Path: {}", path);
                println!("Address: {}", address);
                Ok(())
            }
            AdvancedCommand::Danger { command } => match command {
                DangerCommand::ViewSecrets { name } => {
                    let keechain = KeeChain::open(name, io::get_password)?;
                    keechain.keychain.secrets(network)?.print();
                    Ok(())
                }
                DangerCommand::Wipe { name } => {
                    if io::ask("Are you really sure? This action is permanent!")? && io::ask("Again, are you really sure? THIS ACTION IS PERMANENT AND YOU MAY LOSE ALL YOUR FUNDS!")? {
                        let keechain = KeeChain::open(name, io::get_password)?;
                        keechain.wipe()?;
                    } else {
                        println!("Aborted.");
                    }
                    Ok(())
                }
            },
        },
        Command::Setting { command } => match command {
            SettingCommand::Rename { name, new_name } => {
                let mut keechain = KeeChain::open(name, io::get_password)?;
                keechain.rename(new_name)
            }
            SettingCommand::ChangePassword { name } => {
                let mut keechain = KeeChain::open(name, io::get_password)?;
                keechain.change_password(io::get_password_with_confirmation)
            }
        },
    }
}

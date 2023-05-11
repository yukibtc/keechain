// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use console::Term;
use keechain_core::bip39::Mnemonic;
use keechain_core::bitcoin::Network;
use keechain_core::types::{BitcoinCore, Descriptors, Electrum, KeeChain, Psbt, Wasabi};
use keechain_core::util::dir;
use keechain_core::Result;

mod cli;
mod types;
mod util;

use self::cli::io;
use self::cli::{AdvancedCommand, Cli, Command, DangerCommand, ExportTypes, SettingCommand};

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    let network: Network = args.network.into();
    let keychain_path: PathBuf = keechain_common::keychains()?;

    match args.command {
        Command::Generate {
            name,
            word_count,
            dice_roll,
        } => {
            let path = dir::get_keychain_file(keychain_path, name)?;
            let keechain = KeeChain::generate(
                path,
                io::get_password_with_confirmation,
                word_count.into(),
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
            println!("{}", keechain.keychain.seed.mnemonic());
            println!("\n################################################################\n");

            Ok(())
        }
        Command::Restore { name } => {
            let path = dir::get_keychain_file(keychain_path, name)?;
            KeeChain::restore(path, io::get_password_with_confirmation, || {
                Ok(Mnemonic::from_str(&io::get_input("Seed")?)?)
            })?;
            Ok(())
        }
        Command::List => {
            let names = dir::get_keychains_list(keychain_path)?;
            for (index, name) in names.iter().enumerate() {
                println!("{}. {name}", index + 1);
            }
            Ok(())
        }
        Command::Identity { name } => {
            let path = dir::get_keychain_file(keychain_path, name)?;
            let keechain = KeeChain::open(path, io::get_password)?;
            let fingerprint = keechain.keychain.identity(network)?;
            println!("Fingerprint: {fingerprint}");
            Ok(())
        }
        Command::Export { export_type } => match export_type {
            ExportTypes::Descriptors { name, account } => {
                let path = dir::get_keychain_file(keychain_path, name)?;
                let keechain = KeeChain::open(path, io::get_password)?;
                let descriptors =
                    Descriptors::new(keechain.keychain.seed(), network, Some(account))?;
                println!("{descriptors:#?}");
                Ok(())
            }
            ExportTypes::BitcoinCore { name, account } => {
                let path = dir::get_keychain_file(keychain_path, name)?;
                let keechain = KeeChain::open(path, io::get_password)?;
                let descriptors =
                    BitcoinCore::new(keechain.keychain.seed(), network, Some(account))?;
                println!("{}", descriptors.to_string());
                Ok(())
            }
            ExportTypes::Electrum {
                name,
                script,
                account,
            } => {
                let path = dir::get_keychain_file(keychain_path, name)?;
                let keechain = KeeChain::open(path, io::get_password)?;
                let electrum_json_wallet = Electrum::new(
                    keechain.keychain.seed(),
                    network,
                    script.into(),
                    Some(account),
                )?;
                let path = electrum_json_wallet.save_to_file(keechain_common::home())?;
                println!("Electrum file exported to {}", path.display());
                Ok(())
            }
            ExportTypes::Wasabi { name } => {
                let path = dir::get_keychain_file(keychain_path, name)?;
                let keechain = KeeChain::open(path, io::get_password)?;
                let wasabi_json_wallet = Wasabi::new(keechain.keychain.seed(), network)?;
                let path = wasabi_json_wallet.save_to_file(keechain_common::home())?;
                println!("Wasabi file exported to {}", path.display());
                Ok(())
            }
        },
        Command::Decode { file } => {
            let psbt = Psbt::from_file(file)?;
            util::print_psbt(psbt, network);
            Ok(())
        }
        Command::Sign { name, file } => {
            let path = dir::get_keychain_file(keychain_path, name)?;
            let keechain = KeeChain::open(path, io::get_password)?;
            let mut psbt = Psbt::from_file(&file)?;
            if psbt.sign(&keechain.keychain.seed(), network)? {
                let mut renamed_file: PathBuf = file;
                dir::rename_psbt_to_signed(&mut renamed_file)?;
                psbt.save_to_file(renamed_file)?;
                println!("Signed.");
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
                let path = dir::get_keychain_file(keychain_path, name)?;
                let keechain = KeeChain::open(path, io::get_password)?;
                let mnemonic: Mnemonic =
                    keechain
                        .keychain
                        .deterministic_entropy(network, word_count.into(), index)?;
                println!("Mnemonic: {mnemonic}");
                Ok(())
            }
            AdvancedCommand::Danger { command } => match command {
                DangerCommand::ViewSecrets { name } => {
                    let path = dir::get_keychain_file(keychain_path, name)?;
                    let keechain = KeeChain::open(path, io::get_password)?;
                    let secrets = keechain.keychain.secrets(network)?;
                    util::print_secrets(secrets);
                    Ok(())
                }
                DangerCommand::Wipe { name } => {
                    if io::ask("Are you really sure? This action is permanent!")? && io::ask("Again, are you really sure? THIS ACTION IS PERMANENT AND YOU MAY LOSE ALL YOUR FUNDS!")? {
                        let path = dir::get_keychain_file(keychain_path, name)?;
                        let keechain = KeeChain::open(path, io::get_password)?;
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
                let path = dir::get_keychain_file(&keychain_path, name)?;
                let mut keechain = KeeChain::open(path, io::get_password)?;
                let new_path = dir::get_keychain_file(keychain_path, new_name)?;
                Ok(keechain.rename(new_path)?)
            }
            SettingCommand::ChangePassword { name } => {
                let path = dir::get_keychain_file(keychain_path, name)?;
                let mut keechain = KeeChain::open(path, io::get_password)?;
                Ok(keechain.change_password(io::get_password_with_confirmation)?)
            }
        },
    }
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use clap::Parser;
use cli::DangerCommands;

mod cli;
mod command;
mod types;
mod util;

use self::cli::{Cli, Commands};
use self::util::io;

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Restore { name } => {
            let password: String = rpassword::prompt_password("Password: ")?;
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
        Commands::Export { name, account } => {
            let password: String = rpassword::prompt_password("Password: ")?;
            command::get_public_keys(name, password, args.network, Some(account))?;
            Ok(())
        }
        Commands::Sign { name, file } => {
            println!("{} - {}", name, file.display());
            Ok(())
        }
        Commands::Danger { command } => match command {
            DangerCommands::ViewSeed { name } => {
                let password: String = rpassword::prompt_password("Password: ")?;
                command::view_seed(name, password)
            }
            DangerCommands::Wipe { name } => {
                let password: String = rpassword::prompt_password("Password: ")?;
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

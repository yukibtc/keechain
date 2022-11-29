// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use console::Term;
use dialoguer::{Confirm, Input, Password, Select};

pub fn get_input<S>(prompt: S) -> Result<String>
where
    S: Into<String>,
{
    Ok(Input::new().with_prompt(prompt).interact_text()?)
}

pub fn get_password() -> Result<String> {
    Ok(Password::new().with_prompt("Password").interact()?)
}

pub fn get_password_with_confirmation() -> Result<String> {
    Ok(Password::new()
        .with_prompt("Password")
        .with_confirmation("Confirm password", "Passwords mismatching")
        .interact()?)
}

pub fn ask<S>(prompt: S) -> Result<bool>
where
    S: Into<String> + std::marker::Copy,
{
    if Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()?
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn select_dice_roll(term: Term, rolls: &mut Vec<u8>) -> Result<()> {
    term.write_line(&format!("Total rolls: {}", rolls.len()))?;
    term.write_line("Select number:")?;
    let items: Vec<&str> = vec!["1", "2", "3", "4", "5", "6", "finish"];
    let index: usize = Select::new().default(0).items(&items).interact()?;
    let value: &str = items[index];
    if let Ok(num) = value.parse::<u8>() {
        rolls.push(num);
        term.clear_last_lines(2)?;
        select_dice_roll(term, rolls)?;
    }
    Ok(())
}

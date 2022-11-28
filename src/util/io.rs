// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use dialoguer::{Confirm, Input, Password};

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
    if Confirm::new().with_prompt(prompt).interact()? {
        Ok(true)
    } else {
        Ok(false)
    }
}

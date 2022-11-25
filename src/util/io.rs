// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::io::{stdin, stdout, Write};

use anyhow::Result;

pub fn get_input<S>(prompt: S) -> Result<String>
where
    S: Into<String>,
{
    let mut input = String::new();
    print!("{}", prompt.into());
    let _ = stdout().flush();
    stdin().read_line(&mut input)?;
    if let Some('\n') | Some('\r') = input.chars().next_back() {
        input.pop();
    }
    Ok(input)
}

pub fn ask<S>(prompt: S) -> Result<bool>
where
    S: Into<String> + std::marker::Copy,
{
    let input: String = get_input(format!("{} [y/N] ", prompt.into()))?;
    match input.as_str() {
        "y" | "yes" | "Y" | "Yes" => Ok(true),
        "n" | "no" | "N" | "No" | "" => Ok(false),
        _ => ask(prompt),
    }
}

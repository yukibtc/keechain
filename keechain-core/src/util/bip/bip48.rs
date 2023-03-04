// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::util::bip32::{ChildNumber, DerivationPath, Error};
use bitcoin::Network;

use super::bip32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ScriptType {
    P2SHWSH = 1,
    P2WSH = 2,
}

impl ScriptType {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

pub fn account_extended_path(
    network: Network,
    account: Option<u32>,
    script_type: ScriptType,
) -> Result<DerivationPath, Error> {
    // Path: m/<purpose>'/<coin_type>'/<account>'/<script_type>'
    let base_path = bip32::account_extended_path(48, network, account)?;
    let path: Vec<ChildNumber> = vec![ChildNumber::from_hardened_idx(script_type.as_u32())?];
    Ok(base_path.extend(path))
}

pub fn extended_path(
    network: Network,
    account: Option<u32>,
    script_type: ScriptType,
    change: bool,
) -> Result<DerivationPath, Error> {
    // Path: m/<purpose>'/<coin>'/<account>'/<script_type>'/<change>
    let base_path = account_extended_path(network, account, script_type)?;
    let path: Vec<ChildNumber> = vec![ChildNumber::from_normal_idx(u32::from(change))?];
    Ok(base_path.extend(path))
}

pub fn get_path(
    network: Network,
    account: Option<u32>,
    script_type: ScriptType,
    change: bool,
    index: Option<u32>,
) -> Result<DerivationPath, Error> {
    // Path: m/<purpose>'/<coin_type>'/<account>'/<script_type>'/<change>/<index>
    let base_path = account_extended_path(network, account, script_type)?;
    let path: Vec<ChildNumber> = vec![
        ChildNumber::from_normal_idx(u32::from(change))?,
        ChildNumber::from_normal_idx(index.unwrap_or(0))?,
    ];
    Ok(base_path.extend(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn test_p2wsh_path() -> Result<()> {
        assert_eq!(
            get_path(Network::Bitcoin, None, ScriptType::P2WSH, false, None)?.to_string(),
            "m/48'/0'/0'/2'/0/0".to_string()
        );

        assert_eq!(
            get_path(Network::Bitcoin, None, ScriptType::P2WSH, false, Some(1))?.to_string(),
            "m/48'/0'/0'/2'/0/1".to_string()
        );

        assert_eq!(
            get_path(Network::Bitcoin, None, ScriptType::P2WSH, true, None)?.to_string(),
            "m/48'/0'/0'/2'/1/0".to_string()
        );

        assert_eq!(
            get_path(Network::Bitcoin, Some(1), ScriptType::P2WSH, false, None)?.to_string(),
            "m/48'/0'/1'/2'/0/0".to_string()
        );

        assert_eq!(
            get_path(Network::Testnet, None, ScriptType::P2WSH, true, Some(5))?.to_string(),
            "m/48'/1'/0'/2'/1/5".to_string()
        );

        Ok(())
    }

    #[test]
    fn test_p2shwsh_path() -> Result<()> {
        assert_eq!(
            get_path(Network::Bitcoin, None, ScriptType::P2SHWSH, false, None)?.to_string(),
            "m/48'/0'/0'/1'/0/0".to_string()
        );

        assert_eq!(
            get_path(Network::Bitcoin, None, ScriptType::P2SHWSH, false, Some(1))?.to_string(),
            "m/48'/0'/0'/1'/0/1".to_string()
        );

        assert_eq!(
            get_path(Network::Bitcoin, None, ScriptType::P2SHWSH, true, None)?.to_string(),
            "m/48'/0'/0'/1'/1/0".to_string()
        );

        assert_eq!(
            get_path(Network::Bitcoin, Some(1), ScriptType::P2SHWSH, false, None)?.to_string(),
            "m/48'/0'/1'/1'/0/0".to_string()
        );

        assert_eq!(
            get_path(Network::Testnet, None, ScriptType::P2SHWSH, true, Some(5))?.to_string(),
            "m/48'/1'/0'/1'/1/5".to_string()
        );

        Ok(())
    }
}

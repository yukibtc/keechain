// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use core::str::FromStr;
use std::collections::HashMap;

use bdk::bitcoin::address::{Address, NetworkUnchecked};
use bdk::bitcoin::Network;
use bdk::miniscript::DescriptorPublicKey;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::bips::bip32::{DerivationPath, ExtendedPubKey, Fingerprint};
use crate::bips::bip43::Purpose;
use crate::bips::bip48::ScriptType;
use crate::descriptors::{self, descriptor};

#[derive(Debug)]
pub enum Error {
    Descriptors(descriptors::Error),
    Json(serde_json::Error),
    UnknownNetwork,
    PurposeNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Descriptors(e) => write!(f, "Descriptors: {e}"),
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::UnknownNetwork => write!(f, "unknown network"),
            Self::PurposeNotFound => write!(f, "purpose not found"),
        }
    }
}

impl From<descriptors::Error> for Error {
    fn from(e: descriptors::Error) -> Self {
        Self::Descriptors(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct ColdcardGenericJsonChild {
    name: String,
    xfp: Fingerprint,
    deriv: DerivationPath,
    xpub: ExtendedPubKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    first: Option<Address<NetworkUnchecked>>,
}

/// Generic JSON (Coldcard format)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColdcardGenericJson {
    chain: ColdcardGenericJsonNetwork,
    xfp: Fingerprint,
    account: u32,
    xpub: ExtendedPubKey,
    #[serde(
        flatten,
        serialize_with = "serialize_bips",
        deserialize_with = "deserialize_bips"
    )]
    #[serde(default)]
    bips: HashMap<Purpose, ColdcardGenericJsonChild>,
}

impl ColdcardGenericJson {
    /* pub fn from_seed(seed: &Seed, account: u32) -> Self {
        todo!()
    } */

    pub fn from_json<T>(json: T) -> Result<Self, Error>
    where
        T: AsRef<[u8]>,
    {
        Ok(serde_json::from_slice(json.as_ref())?)
    }

    pub fn network(&self) -> Network {
        self.chain.into()
    }

    /// Root [`Fingerprint`]`
    pub fn fingerprint(&self) -> Fingerprint {
        self.xfp
    }

    pub fn account(&self) -> u32 {
        self.account
    }

    pub fn bip32_root_pubkey(&self) -> ExtendedPubKey {
        self.xpub
    }

    pub fn descriptor(&self, purpose: Purpose) -> Result<DescriptorPublicKey, Error> {
        let child = self.bips.get(&purpose).ok_or(Error::PurposeNotFound)?;
        let (_, desc) = descriptor(self.xfp, child.xpub, &child.deriv, false)?;
        Ok(desc)
    }

    /* pub fn as_json(&self) -> String {
        serde_json::json!(self).to_string()
    } */
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ColdcardGenericJsonNetwork {
    /// Mainnet
    Btc,
    /// Testnet
    Xtn,
    /// Regtest
    Xrt,
}

impl fmt::Display for ColdcardGenericJsonNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Btc => write!(f, "BTC"),
            Self::Xtn => write!(f, "XTN"),
            Self::Xrt => write!(f, "XRT"),
        }
    }
}

impl FromStr for ColdcardGenericJsonNetwork {
    type Err = Error;
    fn from_str(network: &str) -> Result<Self, Self::Err> {
        match network {
            "BTC" => Ok(Self::Btc),
            "XTN" => Ok(Self::Xtn),
            "XRT" => Ok(Self::Xrt),
            _ => Err(Error::UnknownNetwork),
        }
    }
}

impl From<ColdcardGenericJsonNetwork> for Network {
    fn from(network: ColdcardGenericJsonNetwork) -> Self {
        match network {
            ColdcardGenericJsonNetwork::Btc => Self::Bitcoin,
            ColdcardGenericJsonNetwork::Xtn => Self::Testnet,
            ColdcardGenericJsonNetwork::Xrt => Self::Regtest,
        }
    }
}

impl Serialize for ColdcardGenericJsonNetwork {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ColdcardGenericJsonNetwork {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let network: String = String::deserialize(deserializer)?;
        Self::from_str(&network).map_err(serde::de::Error::custom)
    }
}

fn serialize_bips<S>(
    bips: &HashMap<Purpose, ColdcardGenericJsonChild>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(bips.len()))?;
    for (purpose, child) in bips.iter() {
        let purpose: &str = match purpose {
            Purpose::BIP44 => "bip44",
            Purpose::BIP49 => "bip49",
            Purpose::BIP84 => "bip84",
            Purpose::BIP86 => "bip86",
            Purpose::BIP48 { script } => match script {
                ScriptType::P2SHWSH => "bip48_1",
                ScriptType::P2WSH => "bip48_2",
                ScriptType::P2TR => "bip48_3",
            },
        };
        map.serialize_entry(purpose, child)?;
    }
    map.end()
}

fn deserialize_bips<'de, D>(
    deserializer: D,
) -> Result<HashMap<Purpose, ColdcardGenericJsonChild>, D::Error>
where
    D: Deserializer<'de>,
{
    struct GenericTagsVisitor;

    impl<'de> Visitor<'de> for GenericTagsVisitor {
        type Value = HashMap<Purpose, ColdcardGenericJsonChild>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map in which the keys are \"bipXX\"")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut bips = HashMap::new();
            while let Some(key) = map.next_key::<&str>()? {
                let purpose: Option<Purpose> = match key {
                    "bip44" => Some(Purpose::BIP44),
                    "bip49" => Some(Purpose::BIP49),
                    "bip84" => Some(Purpose::BIP84),
                    "bip86" => Some(Purpose::BIP86),
                    "bip48_1" => Some(Purpose::BIP48 {
                        script: ScriptType::P2SHWSH,
                    }),
                    "bip48_2" => Some(Purpose::BIP48 {
                        script: ScriptType::P2WSH,
                    }),
                    "bip48_3" => Some(Purpose::BIP48 {
                        script: ScriptType::P2TR,
                    }),
                    _ => None,
                };

                match purpose {
                    Some(purpose) => {
                        let child = map.next_value()?;
                        bips.insert(purpose, child);
                    }
                    None => {
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                };
            }
            Ok(bips)
        }
    }

    deserializer.deserialize_map(GenericTagsVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_json_deserialization() {
        let json = r#"{"chain": "XTN", "xfp": "0F056943", "account": 0, "xpub": "tpubD6NzVbkrYhZ4XzL5Dhayo67Gorv1YMS7j8pRUvVMd5odC2LBPLAygka9p7748JtSq82FNGPppFEz5xxZUdasBRCqJqXvUHq6xpnsMcYJzeh", "bip44": {"name": "p2pkh", "xfp": "92B53FD2", "deriv": "m/44'/1'/0'", "xpub": "tpubDCiHGUNYdRRBPNYm7CqeeLwPWfeb2ZT2rPsk4aEW3eUoJM93jbBa7hPpB1T9YKtigmjpxHrB1522kSsTxGm9V6cqKqrp1EDaYaeJZqcirYB", "desc": "pkh([0f056943/44h/1h/0h]tpubDCiHGUNYdRRBPNYm7CqeeLwPWfeb2ZT2rPsk4aEW3eUoJM93jbBa7hPpB1T9YKtigmjpxHrB1522kSsTxGm9V6cqKqrp1EDaYaeJZqcirYB/<0;1>/*)#gx9efxnj", "first": "mtHSVByP9EYZmB26jASDdPVm19gvpecb5R"}, "bip49": {"name": "p2sh-p2wpkh", "xfp": "FD3E8548", "deriv": "m/49'/1'/0'", "xpub": "tpubDCDqt7XXvhAYY9HSwrCXB7BXqYM4RXB8WFtKgtTXGa6u3U6EV1NJJRFTcuTRyhSY5Vreg1LP8aPdyiAPQGrDJLikkHoc7VQg6DA9NtUxHtj", "desc": "sh(wpkh([0f056943/49h/1h/0h]tpubDCDqt7XXvhAYY9HSwrCXB7BXqYM4RXB8WFtKgtTXGa6u3U6EV1NJJRFTcuTRyhSY5Vreg1LP8aPdyiAPQGrDJLikkHoc7VQg6DA9NtUxHtj/<0;1>/*))#7trzzmgc", "_pub": "upub5DMRSsh6mNaeiTXEzarZLvZezWp4cGhaDHjMz9iineDN8syqep2XHncDKFVtTUXY4fyKp12qDVVwdfq5rKkw2CDf5fy2gEHyh5NoTC6fiwm", "first": "2NCAJ5wD4GvmW32GFLVybKPNphNU8UYoEJv"}, "bip84": {"name": "p2wpkh", "xfp": "AB82D43E", "deriv": "m/84'/1'/0'", "xpub": "tpubDC7jGaaSE66Pn4dgtbAAstde4bCyhSUs4r3P8WhMVvPByvcRrzrwqSvpF9Ghx83Z1LfVugGRrSBko5UEKELCz9HoMv5qKmGq3fqnnbS5E9r", "desc": "wpkh([0f056943/84h/1h/0h]tpubDC7jGaaSE66Pn4dgtbAAstde4bCyhSUs4r3P8WhMVvPByvcRrzrwqSvpF9Ghx83Z1LfVugGRrSBko5UEKELCz9HoMv5qKmGq3fqnnbS5E9r/<0;1>/*)#sjuyyvve", "_pub": "vpub5Y5a91QvDT3yog4bmgbqFo7GPXpRpozogzQeDArSPzsY8SKGHTgjSswhxhGkRonUQ9tyo9ZSQ1ecLKkVUyewWEUJZdwgUQycvG86FV7sdhZ", "first": "tb1qupyd58ndsh7lut0et0vtrq432jvu9jtdyws9n9"}, "bip86": {"name": "p2tr", "xfp": "4A29873A", "deriv": "m/86'/1'/0'", "xpub": "tpubDCeEX49avtiXrBTv3JWTtco99Ka499jXdZHBRtm7va2gkMAui11ctZjqNAT9dLVNaEozt2C1kfTM88cnvZCXsWLJN2p4viGvsyGjtKVV7A1", "desc": "tr([0f056943/86h/1h/0h]tpubDCeEX49avtiXrBTv3JWTtco99Ka499jXdZHBRtm7va2gkMAui11ctZjqNAT9dLVNaEozt2C1kfTM88cnvZCXsWLJN2p4viGvsyGjtKVV7A1/<0;1>/*)#e0pwumnv", "first": "tb1prlna6c6us6jss2qyemcm8jpzjpuuyx46tz6pe80r6jmpf5dm3z7qnxwucf"}, "bip48_1": {"name": "p2sh-p2wsh", "xfp": "43BD4CE2", "deriv": "m/48'/1'/0'/1'", "xpub": "tpubDF2rnouQaaYrUEy2JM1YD3RFzew4onawGM4X2Re67gguTf5CbHonBRiFGe3Xjz7DK88dxBFGf2i7K1hef3PM4cFKyUjcbJXddaY9F5tJBoP", "desc": "sh(wsh(sortedmulti(M,[0f056943/48'/1'/0'/1']tpubDF2rnouQaaYrUEy2JM1YD3RFzew4onawGM4X2Re67gguTf5CbHonBRiFGe3Xjz7DK88dxBFGf2i7K1hef3PM4cFKyUjcbJXddaY9F5tJBoP/0/*,...)))", "_pub": "Upub5T4XUooQzDXL58NCHk8ZCw9BsRSLCtnyHeZEExAq1XdnBFXiXVrHFuvvmh3TnCR7XmKHxkwqdACv68z7QKT1vwru9L1SZSsw8B2fuBvtSa6"}, "bip48_2": {"name": "p2wsh", "xfp": "B5EE2F16", "deriv": "m/48'/1'/0'/2'", "xpub": "tpubDF2rnouQaaYrXF4noGTv6rQYmx87cQ4GrUdhpvXkhtChwQPbdGTi8GA88NUaSrwZBwNsTkC9bFkkC8vDyGBVVAQTZ2AS6gs68RQXtXcCvkP", "desc": "wsh(sortedmulti(M,[0f056943/48'/1'/0'/2']tpubDF2rnouQaaYrXF4noGTv6rQYmx87cQ4GrUdhpvXkhtChwQPbdGTi8GA88NUaSrwZBwNsTkC9bFkkC8vDyGBVVAQTZ2AS6gs68RQXtXcCvkP/0/*,...))", "_pub": "Vpub5mtnnUUL8u4oyRf5d2NZJqDypgmpx8FontedpqxNyjXTi6fLp8fmpp2wedS6UyuNpDgLDoVH23c6rYpFSEfB9jhdbD8gek2stjxhwJeE1Eq"}, "bip48_3": {"name": "p2tr", "xfp": "404EEEE5", "deriv": "m/48'/1'/0'/3'", "xpub": "tpubDF2rnouQaaYrY6CUWTapYkeFEs3h3qrzL4M52ZGoPeU9dkarJMtrw6VF1zJRGuGuAFxYS3kXtavfAwQPTQkU5dyNYpbgxcpftrR8H3U85Ez", "desc": "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(M,[0f056943/48'/1'/0'/3']tpubDF2rnouQaaYrY6CUWTapYkeFEs3h3qrzL4M52ZGoPeU9dkarJMtrw6VF1zJRGuGuAFxYS3kXtavfAwQPTQkU5dyNYpbgxcpftrR8H3U85Ez/0/*,...))"}, "bip45": {"name": "p2sh", "xfp": "9222584E", "deriv": "m/45'", "xpub": "tpubD8NXmKsmWp3a3DXhbihAYbYLGaRNVdTnr6JoSxxfXYQcmwVtW2hv8QoDwng6JtEonmJoL3cNEwfd2cLXMpGezwZ2vL2dQ7259bueNKj9C8n", "desc": "sh(sortedmulti(M,[0f056943/45']tpubD8NXmKsmWp3a3DXhbihAYbYLGaRNVdTnr6JoSxxfXYQcmwVtW2hv8QoDwng6JtEonmJoL3cNEwfd2cLXMpGezwZ2vL2dQ7259bueNKj9C8n/0/*,...))"}}"#;
        let generic_json = ColdcardGenericJson::from_json(json).unwrap();

        // Check network
        assert_eq!(generic_json.chain, ColdcardGenericJsonNetwork::Xtn);
        assert_eq!(generic_json.network(), Network::Testnet);

        // Check descriptors
        assert_eq!(generic_json.descriptor(Purpose::BIP44).unwrap(), DescriptorPublicKey::from_str("[0f056943/44'/1'/0']tpubDCiHGUNYdRRBPNYm7CqeeLwPWfeb2ZT2rPsk4aEW3eUoJM93jbBa7hPpB1T9YKtigmjpxHrB1522kSsTxGm9V6cqKqrp1EDaYaeJZqcirYB/0/*").unwrap());
        assert_eq!(generic_json.descriptor(Purpose::BIP49).unwrap(), DescriptorPublicKey::from_str("[0f056943/49'/1'/0']tpubDCDqt7XXvhAYY9HSwrCXB7BXqYM4RXB8WFtKgtTXGa6u3U6EV1NJJRFTcuTRyhSY5Vreg1LP8aPdyiAPQGrDJLikkHoc7VQg6DA9NtUxHtj/0/*").unwrap());
        assert_eq!(generic_json.descriptor(Purpose::BIP84).unwrap(), DescriptorPublicKey::from_str("[0f056943/84'/1'/0']tpubDC7jGaaSE66Pn4dgtbAAstde4bCyhSUs4r3P8WhMVvPByvcRrzrwqSvpF9Ghx83Z1LfVugGRrSBko5UEKELCz9HoMv5qKmGq3fqnnbS5E9r/0/*").unwrap());
        assert_eq!(generic_json.descriptor(Purpose::BIP86).unwrap(), DescriptorPublicKey::from_str("[0f056943/86'/1'/0']tpubDCeEX49avtiXrBTv3JWTtco99Ka499jXdZHBRtm7va2gkMAui11ctZjqNAT9dLVNaEozt2C1kfTM88cnvZCXsWLJN2p4viGvsyGjtKVV7A1/0/*").unwrap());
        assert_eq!(generic_json.descriptor(Purpose::BIP48 { script: ScriptType::P2SHWSH }).unwrap(), DescriptorPublicKey::from_str("[0f056943/48'/1'/0'/1']tpubDF2rnouQaaYrUEy2JM1YD3RFzew4onawGM4X2Re67gguTf5CbHonBRiFGe3Xjz7DK88dxBFGf2i7K1hef3PM4cFKyUjcbJXddaY9F5tJBoP/0/*").unwrap());
        assert_eq!(generic_json.descriptor(Purpose::BIP48 { script: ScriptType::P2WSH }).unwrap(), DescriptorPublicKey::from_str("[0f056943/48'/1'/0'/2']tpubDF2rnouQaaYrXF4noGTv6rQYmx87cQ4GrUdhpvXkhtChwQPbdGTi8GA88NUaSrwZBwNsTkC9bFkkC8vDyGBVVAQTZ2AS6gs68RQXtXcCvkP/0/*").unwrap());
        assert_eq!(generic_json.descriptor(Purpose::BIP48 { script: ScriptType::P2TR }).unwrap(), DescriptorPublicKey::from_str("[0f056943/48'/1'/0'/3']tpubDF2rnouQaaYrY6CUWTapYkeFEs3h3qrzL4M52ZGoPeU9dkarJMtrw6VF1zJRGuGuAFxYS3kXtavfAwQPTQkU5dyNYpbgxcpftrR8H3U85Ez/0/*").unwrap());
    }
}

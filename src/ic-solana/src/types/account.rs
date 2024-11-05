use std::str::FromStr;

use base64::{prelude::BASE64_STANDARD, Engine};
use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::types::{pubkey::Pubkey, CandidValue, Epoch};

/// An Account with data that is stored on a chain
#[derive(PartialEq, Eq, Clone, Default, Debug, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// lamports in the account
    pub lamports: u64,
    /// data held in this account
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    /// The program that owns this account. If executable, the program that loads this account.
    pub owner: Pubkey,
    /// this account's data contains a loaded program (and is now read-only)
    pub executable: bool,
    /// the epoch at which this account will next owe rent
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: Epoch,
}

/// A duplicate representation of an Account for pretty JSON serialization
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiAccount {
    pub lamports: u64,
    pub data: UiAccountData,
    pub owner: String,
    pub executable: bool,
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: Epoch,
    pub space: Option<u64>,
}

impl UiAccount {
    pub fn decode(&self) -> Option<Account> {
        let data = self.data.decode()?;
        Some(Account {
            lamports: self.lamports,
            data,
            owner: Pubkey::from_str(&self.owner).ok()?,
            executable: self.executable,
            rent_epoch: self.rent_epoch,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase", untagged)]
pub enum UiAccountData {
    LegacyBinary(String), // Legacy. Retained for RPC backwards compatibility
    Json(ParsedAccount),
    Binary(String, UiAccountEncoding),
}

impl UiAccountData {
    /// Returns decoded account data in binary format if possible
    pub fn decode(&self) -> Option<Vec<u8>> {
        match self {
            UiAccountData::Json(_) => None,
            UiAccountData::LegacyBinary(blob) => bs58::decode(blob).into_vec().ok(),
            UiAccountData::Binary(blob, encoding) => match encoding {
                UiAccountEncoding::Base58 => bs58::decode(blob).into_vec().ok(),
                UiAccountEncoding::Base64 => BASE64_STANDARD.decode(blob).ok(),
                UiAccountEncoding::Base64Zstd | UiAccountEncoding::Binary | UiAccountEncoding::JsonParsed => None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum UiAccountEncoding {
    #[serde(rename = "binary")]
    Binary, // Legacy. Retained for RPC backwards compatibility
    #[serde(rename = "base58")]
    Base58,
    #[serde(rename = "base64")]
    #[default]
    Base64,
    #[serde(rename = "jsonParsed")]
    JsonParsed,
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct ParsedAccount {
    pub program: String,
    pub parsed: CandidValue,
    pub space: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct AccountKey {
    pub pubkey: String,
    pub writable: bool,
    pub signer: bool,
    pub source: Option<AccountKeySource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum AccountKeySource {
    Transaction,
    LookupTable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiTokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

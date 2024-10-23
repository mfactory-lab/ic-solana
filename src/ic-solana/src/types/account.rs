use {
    super::CandidValue,
    crate::types::{pubkey::Pubkey, Epoch},
    base64::{prelude::BASE64_STANDARD, Engine},
    candid::{CandidType, Deserialize},
    serde::Serialize,
    std::str::FromStr,
};

/// An Account with data that is stored on a chain
#[derive(Deserialize, PartialEq, Eq, Clone, Default, CandidType, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum UiAccountEncoding {
    Binary, // Legacy. Retained for RPC backwards compatibility
    Base58,
    Base64,
    JsonParsed,
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct ParsedAccount {
    pub program: String,
    pub parsed: CandidValue,
    pub space: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiTokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

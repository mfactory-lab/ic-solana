use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::{
    account::UiAccountEncoding,
    commitment::CommitmentLevel,
    filter::RpcFilterType,
    response::RpcBlockProductionRange,
    transaction::{TransactionDetails, UiTransactionEncoding},
    Epoch, Slot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiDataSliceConfig {
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSignatureStatusConfig {
    #[serde(rename = "searchTransactionHistory")]
    pub search_transaction_history: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSendTransactionConfig {
    #[serde(default)]
    /// When true, skip the preflight transaction checks.
    /// Default: false
    #[serde(rename = "skipPreflight")]
    pub skip_preflight: bool,
    /// Commitment level to use for preflight.
    /// Default: `Finalized`
    #[serde(rename = "preflightCommitment")]
    pub preflight_commitment: Option<CommitmentLevel>,
    /// Encoding used for the transaction data.
    /// Default: `Base64`
    pub encoding: Option<UiTransactionEncoding>,
    /// Maximum number of times for the RPC node to retry sending the transaction to the leader.
    /// If this parameter is not provided, the RPC node will retry the transaction until it is
    /// finalized or until the blockhash expires.
    #[serde(rename = "maxRetries")]
    pub max_retries: Option<usize>,
    /// Set the minimum slot at which to perform preflight transaction checks.
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<Slot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSimulateTransactionAccountsConfig {
    pub encoding: Option<UiAccountEncoding>,
    pub addresses: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSimulateTransactionConfig {
    #[serde(default, rename = "sigVerify")]
    pub sig_verify: bool,
    #[serde(default, rename = "replaceRecentBlockhash")]
    pub replace_recent_blockhash: bool,
    pub commitment: Option<CommitmentLevel>,
    pub encoding: Option<UiTransactionEncoding>,
    pub accounts: Option<RpcSimulateTransactionAccountsConfig>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<Slot>,
    #[serde(default, rename = "innerInstructions")]
    pub inner_instructions: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcRequestAirdropConfig {
    #[serde(rename = "recentBlockhash")]
    pub recent_blockhash: Option<String>, // base-58 encoded blockhash
    pub commitment: Option<CommitmentLevel>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcLeaderScheduleConfig {
    pub identity: Option<String>, // validator identity, as a base-58 encoded string
    pub commitment: Option<CommitmentLevel>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockProductionConfig {
    pub identity: Option<String>, // validator identity, as a base-58 encoded string
    pub range: Option<RpcBlockProductionRange>, // current epoch if `None`
    pub commitment: Option<CommitmentLevel>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcGetVoteAccountsConfig {
    #[serde(rename = "votePubkey")]
    pub vote_pubkey: Option<String>, // validator vote address, as a base-58 encoded string
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "keepUnstakedDelinquents")]
    pub keep_unstaked_delinquents: Option<bool>,
    #[serde(rename = "delinquentSlotDistance")]
    pub delinquent_slot_distance: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcLeaderScheduleConfigWrapper {
    SlotOnly(Option<Slot>),
    ConfigOnly(Option<RpcLeaderScheduleConfig>),
}

impl RpcLeaderScheduleConfigWrapper {
    pub fn unzip(&self) -> (Option<Slot>, Option<RpcLeaderScheduleConfig>) {
        match &self {
            RpcLeaderScheduleConfigWrapper::SlotOnly(slot) => (*slot, None),
            RpcLeaderScheduleConfigWrapper::ConfigOnly(config) => (None, config.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum RpcLargestAccountsFilter {
    Circulating,
    NonCirculating,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcLargestAccountsConfig {
    pub commitment: Option<CommitmentLevel>,
    pub filter: Option<RpcLargestAccountsFilter>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSupplyConfig {
    pub commitment: Option<CommitmentLevel>,
    #[serde(default, rename = "excludeNonCirculatingAccountsList")]
    pub exclude_non_circulating_accounts_list: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcEpochConfig {
    pub epoch: Option<Epoch>,
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<Slot>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum RpcAccountIndex {
    ProgramId,
    SplTokenMint,
    SplTokenOwner,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccountInfoConfig {
    pub encoding: Option<UiAccountEncoding>,
    #[serde(rename = "dataSlice")]
    pub data_slice: Option<UiDataSliceConfig>,
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<Slot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcProgramAccountsConfig {
    pub filters: Option<Vec<RpcFilterType>>,
    // #[serde(flatten)]
    // pub account_config: RpcAccountInfoConfig,
    pub encoding: Option<UiAccountEncoding>,
    #[serde(rename = "dataSlice")]
    pub data_slice: Option<UiDataSliceConfig>,
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<Slot>,
    #[serde(rename = "withContext")]
    pub with_context: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RpcTransactionLogsFilter {
    All,
    AllWithVotes,
    Mentions(Vec<String>), // base58-encoded list of addresses
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionLogsConfig {
    pub commitment: Option<CommitmentLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum RpcTokenAccountsFilter {
    #[serde(rename = "mint")]
    Mint(String),
    #[serde(rename = "programId")]
    ProgramId(String),
}

// #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct RpcSignatureSubscribeConfig {
//     pub commitment: Option<CommitmentLevel>,
//     #[serde(rename = "enableReceivedNotification")]
//     pub enable_received_notification: Option<bool>,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub enum RpcBlockSubscribeFilter {
//     All,
//     MentionsAccountOrProgram(String),
// }

// #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct RpcBlockSubscribeConfig {
//     pub commitment: Option<CommitmentLevel>,
//     pub encoding: Option<UiTransactionEncoding>,
//     pub transaction_details: Option<TransactionDetails>,
//     pub show_rewards: Option<bool>,
//     pub max_supported_transaction_version: Option<u8>,
// }

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSignaturesForAddressConfig {
    pub before: Option<String>, // Signature as base-58 string
    pub until: Option<String>,  // Signature as base-58 string
    pub limit: Option<usize>,
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<Slot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcEncodingConfigWrapper<T> {
    Deprecated(Option<UiTransactionEncoding>),
    Current(Option<T>),
}

impl<T: EncodingConfig + Default + Copy> RpcEncodingConfigWrapper<T> {
    pub fn convert_to_current(&self) -> T {
        match self {
            RpcEncodingConfigWrapper::Deprecated(encoding) => T::new_with_encoding(encoding),
            RpcEncodingConfigWrapper::Current(config) => config.unwrap_or_default(),
        }
    }

    pub fn convert<U: EncodingConfig + From<T>>(&self) -> RpcEncodingConfigWrapper<U> {
        match self {
            RpcEncodingConfigWrapper::Deprecated(encoding) => RpcEncodingConfigWrapper::Deprecated(*encoding),
            RpcEncodingConfigWrapper::Current(config) => {
                RpcEncodingConfigWrapper::Current(config.map(|config| config.into()))
            }
        }
    }
}

pub trait EncodingConfig {
    fn new_with_encoding(encoding: &Option<UiTransactionEncoding>) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockConfig {
    pub encoding: Option<UiTransactionEncoding>,
    #[serde(rename = "transactionDetails")]
    pub transaction_details: Option<TransactionDetails>,
    pub rewards: Option<bool>,
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
}

impl Default for RpcBlockConfig {
    fn default() -> Self {
        Self {
            encoding: None,
            transaction_details: None,
            rewards: None,
            commitment: None,
            max_supported_transaction_version: Some(0),
        }
    }
}

impl EncodingConfig for RpcBlockConfig {
    fn new_with_encoding(encoding: &Option<UiTransactionEncoding>) -> Self {
        Self {
            encoding: *encoding,
            ..Self::default()
        }
    }
}

impl RpcBlockConfig {
    pub fn rewards_only() -> Self {
        Self {
            transaction_details: Some(TransactionDetails::None),
            ..Self::default()
        }
    }

    pub fn rewards_with_commitment(commitment: Option<CommitmentLevel>) -> Self {
        Self {
            transaction_details: Some(TransactionDetails::None),
            commitment,
            ..Self::default()
        }
    }
}

impl From<RpcBlockConfig> for RpcEncodingConfigWrapper<RpcBlockConfig> {
    fn from(config: RpcBlockConfig) -> Self {
        RpcEncodingConfigWrapper::Current(Some(config))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionConfig {
    pub encoding: Option<UiTransactionEncoding>,
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
}

impl Default for RpcTransactionConfig {
    fn default() -> Self {
        Self {
            encoding: None,
            commitment: None,
            max_supported_transaction_version: Some(0),
        }
    }
}

impl EncodingConfig for RpcTransactionConfig {
    fn new_with_encoding(encoding: &Option<UiTransactionEncoding>) -> Self {
        Self {
            encoding: *encoding,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcBlocksConfigWrapper {
    EndSlotOnly(Option<Slot>),
    ConfigOnly(Option<RpcContextConfig>),
}

impl RpcBlocksConfigWrapper {
    pub fn unzip(&self) -> (Option<Slot>, Option<RpcContextConfig>) {
        match &self {
            RpcBlocksConfigWrapper::EndSlotOnly(end_slot) => (*end_slot, None),
            RpcBlocksConfigWrapper::ConfigOnly(config) => (None, *config),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcContextConfig {
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<Slot>,
}

use std::{collections::HashMap, fmt};

use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::{
    EncodedTransactionWithStatusMeta, Epoch, Rewards, Slot, TransactionConfirmationStatus, TransactionError, UiAccount,
    UiInnerInstructions, UiTokenAmount, UiTransactionReturnData, UnixTimestamp,
};

/// Wrapper for rpc returns types of methods that provide responses both with and without context.
/// The Main purpose of this is to fix methods that lack context information in their return type
/// without breaking backwards compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalContext<T> {
    Context(RpcResponse<T>),
    NoContext(T),
}

impl<T> OptionalContext<T> {
    pub fn parse_value(self) -> T {
        match self {
            Self::Context(response) => response.value,
            Self::NoContext(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponseContext {
    pub slot: Slot,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
}

impl RpcResponseContext {
    pub fn new(slot: Slot) -> Self {
        Self {
            slot,
            api_version: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub context: RpcResponseContext,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockCommitment<T = [u64; 32]> {
    pub commitment: Option<T>,
    #[serde(rename = "totalStake")]
    pub total_stake: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockhash {
    pub blockhash: String,
    #[serde(rename = "lastValidBlockHeight")]
    pub last_valid_block_height: u64,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcInflationGovernor {
    pub initial: f64,
    pub terminal: f64,
    pub taper: f64,
    pub foundation: f64,
    #[serde(rename = "foundationTerm")]
    pub foundation_term: f64,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcInflationRate {
    pub total: f64,
    pub validator: f64,
    pub foundation: f64,
    pub epoch: Epoch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcKeyedAccount {
    pub pubkey: String,
    pub account: UiAccount,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcContactInfo {
    /// Pubkey of the node as a base-58 string
    pub pubkey: String,
    /// Gossip port
    pub gossip: Option<String>, // Option<SocketAddr>,
    /// Tpu UDP port
    pub tpu: Option<String>,
    /// Tpu QUIC port
    pub tpu_quic: Option<String>,
    /// JSON RPC port
    pub rpc: Option<String>,
    /// WebSocket PubSub port
    pub pubsub: Option<String>,
    /// Software version
    pub version: Option<String>,
    /// First 4 bytes of the FeatureSet identifier
    pub feature_set: Option<u32>,
    /// Shred version
    pub shred_version: Option<u16>,
}

/// Map of leader base58 identity pubkeys to the slot indices relative to the first epoch slot
pub type RpcLeaderSchedule = HashMap<String, Vec<usize>>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockProductionRange {
    #[serde(rename = "firstSlot")]
    pub first_slot: Slot,
    #[serde(rename = "lastSlot", skip_serializing_if = "Option::is_none")]
    pub last_slot: Option<Slot>,
}
//
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockProduction {
    /// Map of leader base58 identity pubkeys to a tuple of `(number of leader slots, number of
    /// blocks produced)`
    #[serde(rename = "byIdentity")]
    pub by_identity: HashMap<String, (usize, usize)>,
    pub range: RpcBlockProductionRange,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "kebab-case")]
pub struct RpcVersionInfo {
    /// The current version of solana-core
    #[serde(rename = "solana-core")]
    pub solana_core: String,
    /// first 4 bytes of the FeatureSet identifier
    #[serde(rename = "feature-set")]
    pub feature_set: Option<u32>,
}

impl fmt::Debug for RpcVersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.solana_core)
    }
}

impl fmt::Display for RpcVersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(version) = self.solana_core.split_whitespace().next() {
            // Display just the semver if possible
            write!(f, "{version}")
        } else {
            write!(f, "{}", self.solana_core)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "kebab-case")]
pub struct RpcIdentity {
    /// The current node identity pubkey
    pub identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcVoteAccountStatus {
    pub current: Vec<RpcVoteAccountInfo>,
    pub delinquent: Vec<RpcVoteAccountInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcVoteAccountInfo {
    /// Vote account address, as base-58 encoded string
    #[serde(rename = "votePubkey")]
    pub vote_pubkey: String,

    /// The validator identity, as base-58 encoded string
    #[serde(rename = "nodePubkey")]
    pub node_pubkey: String,

    /// The current stake, in lamports, delegated to this vote account
    #[serde(rename = "activatedStake")]
    pub activated_stake: u64,

    /// An 8-bit integer used as a fraction (commission/MAX_U8) for rewards payout
    pub commission: u8,

    /// Whether this account is staked for the current epoch
    #[serde(rename = "epochVoteAccount")]
    pub epoch_vote_account: bool,

    /// Latest history of earned credits for up to
    /// `MAX_RPC_VOTE_ACCOUNT_INFO_EPOCH_CREDITS_HISTORY` epochs   each tuple is (Epoch,
    /// credits, prev_credits)
    #[serde(rename = "epochCredits")]
    pub epoch_credits: Vec<(Epoch, u64, u64)>,

    /// The most recent slot voted on by this vote account (0 if no votes exist)
    #[serde(rename = "lastVote")]
    pub last_vote: u64,

    /// Current root slot for this vote account (0 if no root slot exists)
    #[serde(rename = "rootSlot")]
    pub root_slot: Slot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSimulateTransactionResult {
    pub err: Option<TransactionError>,
    pub logs: Option<Vec<String>>,
    pub accounts: Option<Vec<Option<UiAccount>>>,
    pub units_consumed: Option<u64>,
    pub return_data: Option<UiTransactionReturnData>,
    pub inner_instructions: Option<Vec<UiInnerInstructions>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccountBalance {
    pub address: String,
    pub lamports: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSupply {
    pub total: u64,
    pub circulating: u64,
    #[serde(rename = "nonCirculating")]
    pub non_circulating: u64,
    #[serde(rename = "nonCirculatingAccounts")]
    pub non_circulating_accounts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StakeActivationState {
    Activating,
    Active,
    Deactivating,
    Inactive,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcStakeActivation {
    pub state: StakeActivationState,
    pub active: u64,
    pub inactive: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodedConfirmedBlock {
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: String,
    pub blockhash: String,
    #[serde(rename = "parentSlot")]
    pub parent_slot: Slot,
    pub transactions: Vec<EncodedTransactionWithStatusMeta>,
    pub rewards: Rewards,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodedConfirmedBlockWithoutTransactions {
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: String,
    pub blockhash: String,
    #[serde(rename = "parentSlot")]
    pub parent_slot: Slot,
    pub signatures: Vec<String>,
    pub rewards: Rewards,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcTokenAccountBalance {
    pub address: String,
    #[serde(flatten)]
    pub amount: UiTokenAmount,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcConfirmedTransactionStatusWithSignature {
    pub signature: String, // base 58 encoded signature
    pub slot: Slot,
    pub err: Option<TransactionError>,
    pub memo: Option<String>,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
    #[serde(rename = "confirmationStatus")]
    pub confirmation_status: Option<TransactionConfirmationStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcPerfSample {
    pub slot: Slot,
    #[serde(rename = "numTransactions")]
    pub num_transactions: u64,
    #[serde(rename = "numNonVoteTransactions")]
    pub num_non_vote_transactions: Option<u64>,
    #[serde(rename = "numSlots")]
    pub num_slots: u64,
    #[serde(rename = "samplePeriodSecs")]
    pub sample_period_secs: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcInflationReward {
    pub epoch: Epoch,
    #[serde(rename = "effectiveSlot")]
    pub effective_slot: Slot,
    pub amount: u64, // lamports
    #[serde(rename = "postBalance")]
    pub post_balance: u64, // lamports
    pub commission: Option<u8>, // Vote an account commission when the reward was credited
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct RpcSnapshotSlotInfo {
    pub full: Slot,
    pub incremental: Option<Slot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcPrioritizationFee {
    pub slot: Slot,
    #[serde(rename = "prioritizationFee")]
    pub prioritization_fee: u64,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct RpcSignatureConfirmation {
//     pub confirmations: usize,
//     pub status: Result<()>,
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct RpcStorageTurn {
//     pub blockhash: String,
//     pub slot: Slot,
// }

// PubSub Client types for future usage
// https://github.com/solana-labs/solana/blob/master/pubsub-client/src/pubsub_client.rs
//

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct RpcVote {
//     /// Vote account address, as base-58 encoded string
//     pub vote_pubkey: String,
//     pub slots: Vec<Slot>,
//     pub hash: String,
//     pub timestamp: Option<UnixTimestamp>,
//     pub signature: String,
// }

// #[derive(Clone, Deserialize, Serialize, Debug, Error, Eq, PartialEq)]
// pub enum RpcBlockUpdateError {
//     #[error("block store error")]
//     BlockStoreError,
//
//     #[error("unsupported transaction version ({0})")]
//     UnsupportedTransactionVersion(u8),
// }

// #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct RpcBlockUpdate {
//     pub slot: Slot,
//     pub block: Option<UiConfirmedBlock>,
//     pub err: Option<RpcBlockUpdateError>,
// }

// #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
// pub struct SlotInfo {
//     pub slot: Slot,
//     pub parent: Slot,
//     pub root: Slot,
// }
//
// #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct SlotTransactionStats {
//     pub num_transaction_entries: u64,
//     pub num_successful_transactions: u64,
//     pub num_failed_transactions: u64,
//     pub max_transactions_per_entry: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
// #[serde(rename_all = "camelCase", tag = "type")]
// pub enum SlotUpdate {
//     FirstShredReceived {
//         slot: Slot,
//         timestamp: u64,
//     },
//     Completed {
//         slot: Slot,
//         timestamp: u64,
//     },
//     CreatedBank {
//         slot: Slot,
//         parent: Slot,
//         timestamp: u64,
//     },
//     Frozen {
//         slot: Slot,
//         timestamp: u64,
//         stats: SlotTransactionStats,
//     },
//     Dead {
//         slot: Slot,
//         timestamp: u64,
//         err: String,
//     },
//     OptimisticConfirmation {
//         slot: Slot,
//         timestamp: u64,
//     },
//     Root {
//         slot: Slot,
//         timestamp: u64,
//     },
// }
//
// impl SlotUpdate {
//     pub fn slot(&self) -> Slot {
//         match self {
//             Self::FirstShredReceived { slot, .. } => *slot,
//             Self::Completed { slot, .. } => *slot,
//             Self::CreatedBank { slot, .. } => *slot,
//             Self::Frozen { slot, .. } => *slot,
//             Self::Dead { slot, .. } => *slot,
//             Self::OptimisticConfirmation { slot, .. } => *slot,
//             Self::Root { slot, .. } => *slot,
//         }
//     }
// }
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase", untagged)]
// pub enum RpcSignatureResult {
//     ProcessedSignature(ProcessedSignatureResult),
//     ReceivedSignature(ReceivedSignatureResult),
// }
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct RpcLogsResponse {
//     pub signature: String, // Signature as base58 string
//     pub err: Option<TransactionError>,
//     pub logs: Vec<String>,
// }
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct ProcessedSignatureResult {
//     pub err: Option<TransactionError>,
// }
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub enum ReceivedSignatureResult {
//     ReceivedSignature,
// }

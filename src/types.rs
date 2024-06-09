pub mod account;
pub mod epoch_info;
pub mod fee_calculator;
pub mod fees_info;
pub mod filter;
mod message;
pub mod pubkey;
pub mod reward;
pub mod signature;
pub mod transaction;
mod transaction_error;

use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Deserialize, CandidType, Clone, Debug, PartialEq, Eq)]
pub struct InitArgs {
    #[serde(rename = "rpcUrl", skip_serializing_if = "Option::is_none")]
    pub rpc_url: Option<String>,
    #[serde(rename = "nodesInSubnet", skip_serializing_if = "Option::is_none")]
    pub nodes_in_subnet: Option<u32>,
}

/// The unit of time a given leader schedule is honored.
///
/// It lasts for some number of [`Slot`]s.
pub type Epoch = u64;

/// The unit of time given to a leader for encoding a block.
///
/// It is some number of _ticks_ long.
pub type Slot = u64;

/// An approximate measure of real-world time.
///
/// Expressed as Unix time (i.e. seconds since the Unix epoch).
pub type UnixTimestamp = i64;

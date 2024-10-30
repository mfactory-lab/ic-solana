use serde::{Deserialize, Serialize};

use crate::types::Slot;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    pub blockhash: [u8; 32],
    pub fee_calculator: FeeCalculator,
    pub last_valid_block_height: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcFees {
    pub blockhash: String,
    pub fee_calculator: FeeCalculator,
    pub last_valid_slot: Slot,
    pub last_valid_block_height: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcFeeCalculator {
    pub fee_calculator: FeeCalculator,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeCalculator {
    /// The current cost of a signature.
    ///
    /// This amount may increase/decrease over time based on a cluster processing load.
    pub lamports_per_signature: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockhashFeeCalculator {
    pub blockhash: String,
    pub fee_calculator: FeeCalculator,
}

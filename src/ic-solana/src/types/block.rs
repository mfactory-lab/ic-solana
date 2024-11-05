use serde::{Deserialize, Serialize};

use super::{EncodedTransactionWithStatusMeta, Rewards, Slot, UnixTimestamp};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfirmedBlock {
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
    pub blockhash: String,
    #[serde(rename = "parentSlot")]
    pub parent_slot: Slot,
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewards: Option<Rewards>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<EncodedTransactionWithStatusMeta>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<String>>,
    #[serde(default, rename = "numRewardPartitions", skip_serializing_if = "Option::is_none")]
    pub num_reward_partitions: Option<u64>,
}

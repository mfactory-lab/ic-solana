use crate::types::{Epoch, Slot};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EpochInfo {
    /// The current epoch
    pub epoch: Epoch,

    /// The current slot, relative to the start of the current epoch
    pub slot_index: u64,

    /// The number of slots in this epoch
    pub slots_in_epoch: u64,

    /// The absolute current slot
    pub absolute_slot: Slot,

    /// The current block height
    pub block_height: u64,

    /// Total number of transactions processed without an error since genesis
    pub transaction_count: Option<u64>,
}

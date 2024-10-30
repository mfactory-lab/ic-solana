use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::{Epoch, Slot};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct EpochInfo {
    /// The current epoch
    pub epoch: Epoch,

    /// The current slot, relative to the start of the current epoch
    #[serde(rename = "slotIndex")]
    pub slot_index: u64,

    /// The number of slots in this epoch
    #[serde(rename = "slotsInEpoch")]
    pub slots_in_epoch: u64,

    /// The absolute current slot
    #[serde(rename = "absoluteSlot")]
    pub absolute_slot: Slot,

    /// The current block height
    #[serde(rename = "blockHeight")]
    pub block_height: u64,

    /// Total number of transactions processed without an error since genesis
    #[serde(rename = "transactionCount")]
    pub transaction_count: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct EpochSchedule {
    /// The maximum number of slots in each epoch.
    #[serde(rename = "slotsPerEpoch")]
    pub slots_per_epoch: u64,

    /// A number of slots before the beginning of an epoch to calculate
    /// a leader schedule for that epoch.
    #[serde(rename = "leaderScheduleSlotOffset")]
    pub leader_schedule_slot_offset: u64,

    /// Whether epochs start short and grow.
    pub warmup: bool,

    /// The first epoch after the warmup period.
    ///
    /// Basically: `log2(slots_per_epoch) - log2(MINIMUM_SLOTS_PER_EPOCH)`.
    #[serde(rename = "firstNormalEpoch")]
    pub first_normal_epoch: Epoch,

    /// The first slot after the warmup period.
    ///
    /// Basically: `MINIMUM_SLOTS_PER_EPOCH * (2.pow(first_normal_epoch) - 1)`.
    #[serde(rename = "firstNormalSlot")]
    pub first_normal_slot: Slot,
}

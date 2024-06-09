use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FeesInfo {
    blockhash: String,
    lastValidSlot: String,
    lastValidBlockHeight: String,
}

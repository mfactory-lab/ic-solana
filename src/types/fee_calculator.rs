use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeCalculator {
    /// The current cost of a signature.
    ///
    /// This amount may increase/decrease over time based on a cluster processing load.
    pub lamports_per_signature: u64,
}

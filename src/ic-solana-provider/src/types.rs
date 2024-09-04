use {
    candid::{CandidType, Deserialize},
    serde::Serialize,
};

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct SendTransactionRequest {
    pub instructions: Vec<String>,
    pub recent_blockhash: Option<String>,
}

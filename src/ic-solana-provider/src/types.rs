use candid::{CandidType, Deserialize};
use ic_solana::types::Instruction;
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct SendTransactionRequest {
    pub instructions: Vec<Instruction>,
    pub recent_blockhash: Option<String>,
}

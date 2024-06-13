use crate::types::pubkey::Pubkey;
use crate::utils::short_vec;
use bincode::serialize;
use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, CandidType)]
pub struct Instruction {
    /// Pubkey of the program that executes this instruction.
    pub program_id: Pubkey,
    /// Metadata describing accounts that should be passed to the program.
    pub accounts: Vec<AccountMeta>,
    /// Opaque data passed to the program for its own interpretation.
    pub data: Vec<u8>,
}

impl Instruction {
    pub fn new_with_bincode<T: Serialize>(
        program_id: Pubkey,
        data: &T,
        accounts: Vec<AccountMeta>,
    ) -> Self {
        let data = serialize(data).unwrap();
        Self {
            program_id,
            accounts,
            data,
        }
    }

    pub fn new_with_bytes(program_id: Pubkey, data: &[u8], accounts: Vec<AccountMeta>) -> Self {
        Self {
            program_id,
            accounts,
            data: data.to_vec(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct AccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl AccountMeta {
    /// Construct metadata for a writable account.
    pub fn new(pubkey: Pubkey, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable: true,
        }
    }

    /// Construct metadata for a read-only account.
    pub fn new_readonly(pubkey: Pubkey, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable: false,
        }
    }
}

/// A compact encoding of an instruction.
///
/// A `CompiledInstruction` is a component of a multi-instruction [`Message`],
/// which is the core of a Solana transaction. It is created during the
/// construction of `Message`. Most users will not interact with it directly.
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompiledInstruction {
    /// Index into the transaction keys array indicating the program account that executes this instruction.
    pub program_id_index: u8,
    /// Ordered indices into the transaction keys array indicating which accounts to pass to the program.
    #[serde(with = "short_vec")]
    pub accounts: Vec<u8>,
    /// The program input data.
    #[serde(with = "short_vec")]
    pub data: Vec<u8>,
}

impl CompiledInstruction {
    pub fn new<T: Serialize>(program_ids_index: u8, data: &T, accounts: Vec<u8>) -> Self {
        let data = serialize(data).unwrap();
        Self {
            program_id_index: program_ids_index,
            accounts,
            data,
        }
    }

    pub fn new_from_raw_parts(program_id_index: u8, data: Vec<u8>, accounts: Vec<u8>) -> Self {
        Self {
            program_id_index,
            accounts,
            data,
        }
    }

    pub fn program_id<'a>(&self, program_ids: &'a [Pubkey]) -> &'a Pubkey {
        &program_ids[self.program_id_index as usize]
    }
}

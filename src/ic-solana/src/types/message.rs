use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{
    types::{
        account::AccountKey,
        blockhash::BlockHash,
        compiled_keys::CompiledKeys,
        instruction::{CompiledInstruction, Instruction},
        pubkey::Pubkey,
        UiCompiledInstruction, UiInstruction,
    },
    utils::short_vec,
};

/// Bit mask that indicates whether a serialized message is versioned.
pub const MESSAGE_VERSION_PREFIX: u8 = 0x80;

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// The message header, identifying signed and read-only `account_keys`.
    pub header: MessageHeader,

    /// All the account keys used by this transaction.
    #[serde(with = "short_vec")]
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<Pubkey>,

    /// The id of a recent ledger entry.
    #[serde(rename = "recentBlockhash")]
    pub recent_blockhash: BlockHash,

    /// Programs that will be executed in sequence and committed in one atomic transaction if all
    /// succeed.
    #[serde(with = "short_vec")]
    pub instructions: Vec<CompiledInstruction>,
    // /// List of address table lookups used to load additional accounts
    // /// for this transaction.
    // #[serde(with = "short_vec")]
    // pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

impl Message {
    pub fn new(instructions: &[Instruction], payer: Option<&Pubkey>) -> Self {
        Self::new_with_blockhash(instructions, payer, &BlockHash::default())
    }

    pub fn new_with_blockhash(instructions: &[Instruction], payer: Option<&Pubkey>, blockhash: &BlockHash) -> Self {
        let compiled_keys = CompiledKeys::compile(instructions, payer.cloned());
        let (header, account_keys) = compiled_keys
            .try_into_message_components()
            .expect("overflow when compiling message keys");
        let instructions = compile_instructions(instructions, &account_keys);
        Self::new_with_compiled_instructions(
            header.num_required_signatures,
            header.num_readonly_signed_accounts,
            header.num_readonly_unsigned_accounts,
            account_keys,
            *blockhash,
            instructions,
        )
    }

    pub fn new_with_compiled_instructions(
        num_required_signatures: u8,
        num_readonly_signed_accounts: u8,
        num_readonly_unsigned_accounts: u8,
        account_keys: Vec<Pubkey>,
        recent_blockhash: BlockHash,
        instructions: Vec<CompiledInstruction>,
    ) -> Self {
        Self {
            header: MessageHeader {
                num_required_signatures,
                num_readonly_signed_accounts,
                num_readonly_unsigned_accounts,
            },
            account_keys,
            recent_blockhash,
            instructions,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn program_id(&self, instruction_index: usize) -> Option<&Pubkey> {
        Some(&self.account_keys[self.instructions.get(instruction_index)?.program_id_index as usize])
    }

    pub fn program_index(&self, instruction_index: usize) -> Option<usize> {
        Some(self.instructions.get(instruction_index)?.program_id_index as usize)
    }

    pub fn is_signer(&self, i: usize) -> bool {
        i < self.header.num_required_signatures as usize
    }

    pub fn signer_keys(&self) -> Vec<&Pubkey> {
        let last_key = self
            .account_keys
            .len()
            .min(self.header.num_required_signatures as usize);
        self.account_keys[..last_key].iter().collect()
    }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone, Copy, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct MessageHeader {
    /// The number of signatures required for this message to be considered
    /// valid. The signers of those signatures must match the first
    /// `num_required_signatures` of [`Message::account_keys`].
    #[serde(rename = "numRequiredSignatures")]
    pub num_required_signatures: u8,

    /// The last `num_readonly_signed_accounts` of the signed keys are read-only
    /// accounts.
    #[serde(rename = "numReadonlySignedAccounts")]
    pub num_readonly_signed_accounts: u8,

    /// The last `num_readonly_unsigned_accounts` of the unsigned keys are
    /// read-only accounts.
    #[serde(rename = "numReadonlyUnsignedAccounts")]
    pub num_readonly_unsigned_accounts: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum UiMessage {
    #[serde(rename = "parsed")]
    Parsed(UiParsedMessage),
    #[serde(rename = "raw")]
    Raw(UiRawMessage),
}

/// Tagged version of UiMessage
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum UiMessageTagged {
    #[serde(rename = "parsed")]
    Parsed(UiParsedMessage),
    #[serde(rename = "raw")]
    Raw(UiRawMessage),
}

impl From<UiMessage> for UiMessageTagged {
    fn from(ui_message: UiMessage) -> Self {
        match ui_message {
            UiMessage::Parsed(parsed) => Self::Parsed(parsed),
            UiMessage::Raw(raw) => Self::Raw(raw),
        }
    }
}

/// A duplicate representation of a Message, in parsed format, for pretty JSON serialization
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiParsedMessage {
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<AccountKey>,
    #[serde(rename = "recentBlockhash")]
    pub recent_blockhash: String,
    pub instructions: Vec<UiInstruction>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "addressTableLookups")]
    pub address_table_lookups: Option<Vec<UiAddressTableLookup>>,
}

/// A duplicate representation of a Message, in raw format, for pretty JSON serialization
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiRawMessage {
    pub header: MessageHeader,
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<String>,
    #[serde(rename = "recentBlockhash")]
    pub recent_blockhash: String,
    pub instructions: Vec<UiCompiledInstruction>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "addressTableLookups")]
    pub address_table_lookups: Option<Vec<UiAddressTableLookup>>,
}

/// A duplicate representation of a MessageAddressTableLookup, in raw format, for pretty JSON
/// serialization
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiAddressTableLookup {
    #[serde(rename = "accountKey")]
    pub account_key: String,
    #[serde(rename = "writableIndexes")]
    pub writable_indexes: Vec<u8>,
    #[serde(rename = "readonlyIndexes")]
    pub readonly_indexes: Vec<u8>,
}

fn position(keys: &[Pubkey], key: &Pubkey) -> u8 {
    keys.iter().position(|k| k == key).unwrap() as u8
}

fn compile_instruction(ix: &Instruction, keys: &[Pubkey]) -> CompiledInstruction {
    let accounts: Vec<_> = ix
        .accounts
        .iter()
        .map(|account_meta| position(keys, &account_meta.pubkey))
        .collect();

    CompiledInstruction {
        program_id_index: position(keys, &ix.program_id),
        data: ix.data.clone(),
        accounts,
    }
}

fn compile_instructions(ixs: &[Instruction], keys: &[Pubkey]) -> Vec<CompiledInstruction> {
    ixs.iter().map(|ix| compile_instruction(ix, keys)).collect()
}

#[cfg(test)]
mod tests {
    use candid::{Decode, Encode};

    use super::*;
    use crate::types::UiParsedMessage;

    #[test]
    fn test_candid_serialize() {
        let msg = UiMessageTagged::Parsed(UiParsedMessage {
            account_keys: vec![],
            recent_blockhash: "".to_string(),
            instructions: vec![],
            address_table_lookups: None,
        });

        let encoded = Encode!(&msg).unwrap();

        let decoded = Decode!(&encoded, UiMessageTagged).unwrap();

        assert_eq!(msg, decoded);
    }
}

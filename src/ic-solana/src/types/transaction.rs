use std::{
    fmt::{self, Display},
    str::FromStr,
};

use candid::CandidType;
use ic_crypto_ed25519::PrivateKey;
use serde::{de::Error, Deserialize, Serialize};

use super::UiInnerInstructions;
use crate::{
    types::{
        account::{AccountKey, UiTokenAmount},
        message::{Message, UiMessage},
        pubkey::Pubkey,
        reward::Rewards,
        signature::Signature,
        transaction_error::TransactionError,
        BlockHash, CommitmentConfig, Slot, UnixTimestamp,
    },
    utils::short_vec,
};

pub type TransactionResult<T> = Result<T, TransactionError>;

#[derive(Debug, PartialEq, Default, Eq, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(with = "short_vec")]
    pub signatures: Vec<Signature>,
    pub message: Message,
}

impl Transaction {
    pub fn new_unsigned(message: Message) -> Self {
        Self {
            signatures: vec![Signature::default(); message.header.num_required_signatures as usize],
            message,
        }
    }

    pub fn data(&self, instruction_index: usize) -> &[u8] {
        &self.message.instructions[instruction_index].data
    }

    pub fn key(&self, instruction_index: usize, accounts_index: usize) -> Option<&Pubkey> {
        self.key_index(instruction_index, accounts_index)
            .and_then(|account_keys_index| self.message.account_keys.get(account_keys_index))
    }

    pub fn signer_key(&self, instruction_index: usize, accounts_index: usize) -> Option<&Pubkey> {
        match self.key_index(instruction_index, accounts_index) {
            None => None,
            Some(signature_index) => {
                if signature_index >= self.signatures.len() {
                    return None;
                }
                self.message.account_keys.get(signature_index)
            }
        }
    }

    pub fn set_latest_blockhash(&mut self, blockhash: &BlockHash) {
        self.message.recent_blockhash = *blockhash;
    }

    /// Return the message containing all data that should be signed.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Return the serialized message data to sign.
    pub fn message_data(&self) -> Vec<u8> {
        self.message().serialize()
    }

    pub fn is_signed(&self) -> bool {
        self.signatures
            .iter()
            .all(|signature| *signature != Signature::default())
    }

    pub fn sign(&mut self, position: usize, signer: &[u8]) {
        let pk = PrivateKey::deserialize_raw(signer).unwrap();
        let signature = Signature(pk.sign_message(&self.message_data()));
        self.add_signature(position, signature)
    }

    pub fn add_signature(&mut self, position: usize, signature: Signature) {
        self.signatures[position] = signature;
    }

    fn key_index(&self, instruction_index: usize, accounts_index: usize) -> Option<usize> {
        self.message
            .instructions
            .get(instruction_index)
            .and_then(|instruction| instruction.accounts.get(accounts_index))
            .map(|&account_keys_index| account_keys_index as usize)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Transaction serialization failed")
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.serialize()).into_string())
    }
}

impl FromStr for Transaction {
    type Err = bincode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| bincode::Error::custom("Transaction deserialization failed"))?;
        bincode::deserialize(&bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum Legacy {
    #[serde(rename = "legacy")]
    Legacy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum TransactionVersion {
    #[serde(rename = "legacy")]
    Legacy(Legacy),
    #[serde(rename = "number")]
    Number(u8),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum TransactionBinaryEncoding {
    #[serde(rename = "base58")]
    Base58,
    #[serde(rename = "base64")]
    Base64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum UiTransactionEncoding {
    #[serde(rename = "binary")]
    Binary, // Legacy. Retained for RPC backwards compatibility
    #[serde(rename = "base58")]
    Base58,
    #[serde(rename = "base64")]
    Base64,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "jsonParsed")]
    JsonParsed,
}

impl Display for UiTransactionEncoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = serde_json::to_value(self).map_err(|_| fmt::Error)?;
        let s = v.as_str().ok_or(fmt::Error)?;
        write!(f, "{s}")
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum TransactionDetails {
    #[default]
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "signatures")]
    Signatures,
    #[serde(rename = "accounts")]
    Accounts,
    #[serde(rename = "none")]
    None,
}

/// A duplicate representation of a Transaction for pretty JSON serialization
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTransaction {
    pub signatures: Vec<String>,
    pub message: UiMessage,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiAccountsList {
    pub signatures: Vec<String>,
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<AccountKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum EncodedTransaction {
    LegacyBinary(String), /* Old way of expressing base-58, retained for RPC backwards
                           * compatibility */
    Binary(String, TransactionBinaryEncoding),
    Json(UiTransaction),
    Accounts(UiAccountsList),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum TransactionConfirmationStatus {
    #[serde(rename = "processed")]
    Processed,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "finalized")]
    Finalized,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStatus {
    pub slot: Slot,
    pub confirmations: Option<usize>,  // None = rooted
    pub status: TransactionResult<()>, // legacy field
    pub err: Option<TransactionError>,
    #[serde(
        default,
        rename = "confirmationStatus",
        skip_serializing_if = "Option::is_none"
    )]
    pub confirmation_status: Option<TransactionConfirmationStatus>,
}

impl TransactionStatus {
    pub fn satisfies_commitment(&self, commitment_config: CommitmentConfig) -> bool {
        if commitment_config.is_finalized() {
            self.confirmations.is_none()
        } else if commitment_config.is_confirmed() {
            if let Some(status) = &self.confirmation_status {
                *status != TransactionConfirmationStatus::Processed
            } else {
                // These fallback cases handle TransactionStatus RPC responses from older software
                self.confirmations.is_some() && self.confirmations.unwrap() > 1
                    || self.confirmations.is_none()
            }
        } else {
            true
        }
    }

    // Returns `confirmation_status`, or if is_none, determine the status from confirmations.
    // Facilitates querying nodes on older software
    pub fn confirmation_status(&self) -> TransactionConfirmationStatus {
        match &self.confirmation_status {
            Some(status) => status.clone(),
            None => {
                if self.confirmations.is_none() {
                    TransactionConfirmationStatus::Finalized
                } else if self.confirmations.unwrap() > 0 {
                    TransactionConfirmationStatus::Confirmed
                } else {
                    TransactionConfirmationStatus::Processed
                }
            }
        }
    }
}

/// A duplicate representation of TransactionStatusMeta with the `err` field
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiTransactionStatusMeta {
    pub err: Option<TransactionError>,
    pub status: TransactionResult<()>, /* This field is deprecated.  See https://github.com/solana-labs/solana/issues/9302 */
    pub fee: u64,
    #[serde(rename = "preBalances")]
    pub pre_balances: Vec<u64>,
    #[serde(rename = "postBalances")]
    pub post_balances: Vec<u64>,
    #[serde(
        default,
        rename = "innerInstructions",
        skip_serializing_if = "Option::is_none"
    )]
    pub inner_instructions: Option<Vec<UiInnerInstructions>>,
    #[serde(
        default,
        rename = "logMessages",
        skip_serializing_if = "Option::is_none"
    )]
    pub log_messages: Option<Vec<String>>,
    #[serde(
        default,
        rename = "preTokenBalances",
        skip_serializing_if = "Option::is_none"
    )]
    pub pre_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    #[serde(
        default,
        rename = "postTokenBalances",
        skip_serializing_if = "Option::is_none"
    )]
    pub post_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewards: Option<Rewards>,
    #[serde(
        default,
        rename = "loadedAddresses",
        skip_serializing_if = "Option::is_none"
    )]
    pub loaded_addresses: Option<UiLoadedAddresses>,
    #[serde(
        default,
        rename = "returnData",
        skip_serializing_if = "Option::is_none"
    )]
    pub return_data: Option<UiTransactionReturnData>,
    #[serde(
        default,
        rename = "computeUnitsConsumed",
        skip_serializing_if = "Option::is_none"
    )]
    pub compute_units_consumed: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiTransactionReturnData {
    #[serde(rename = "programId")]
    pub program_id: String,
    pub data: (String, UiReturnDataEncoding),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, PartialEq, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum UiReturnDataEncoding {
    #[serde(rename = "base64")]
    Base64,
}

/// A duplicate representation of LoadedAddresses
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiLoadedAddresses {
    pub writable: Vec<String>,
    pub readonly: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiTransactionTokenBalance {
    #[serde(rename = "accountIndex")]
    pub account_index: u8,
    pub mint: String,
    #[serde(rename = "uiTokenAmount")]
    pub ui_token_amount: UiTokenAmount,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, rename = "programId", skip_serializing_if = "Option::is_none")]
    pub program_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodedTransactionWithStatusMeta {
    pub transaction: EncodedTransaction,
    pub meta: Option<UiTransactionStatusMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<TransactionVersion>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodedConfirmedTransactionWithStatusMeta {
    pub slot: Slot,
    #[serde(flatten)]
    pub transaction: EncodedTransactionWithStatusMeta,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
}

#[cfg(test)]
mod tests {
    use bincode::{deserialize, serialize};

    use super::*;
    use crate::types::{
        blockhash::BlockHash,
        instruction::{AccountMeta, Instruction},
        UiParsedMessage,
    };

    fn create_sample_transaction() -> Transaction {
        let pk = PrivateKey::deserialize_raw(&[
            255, 101, 36, 24, 124, 23, 167, 21, 132, 204, 155, 5, 185, 58, 121, 75, 156, 227, 116,
            193, 215, 38, 142, 22, 8, 14, 229, 239, 119, 93, 5, 218,
        ])
        .unwrap();

        let pubkey = Pubkey::from(pk.public_key().serialize_raw());

        let to = Pubkey::from([
            1, 1, 1, 4, 5, 6, 7, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 7, 6, 5, 4,
            1, 1, 1,
        ]);

        let program_id = Pubkey::from([
            2, 2, 2, 4, 5, 6, 7, 8, 9, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9, 8, 7, 6, 5, 4,
            2, 2, 2,
        ]);
        let account_metas = vec![AccountMeta::new(pubkey, true), AccountMeta::new(to, false)];
        let instruction =
            Instruction::new_with_bincode(program_id, &(1u8, 2u8, 3u8), account_metas);

        let message = Message::new_with_blockhash(&[instruction], None, &BlockHash::default());

        let mut tx = Transaction::new_unsigned(message);
        tx.sign(0, &pk.serialize_raw());

        tx
    }

    #[test]
    fn test_transaction_serialize() {
        let tx = create_sample_transaction();
        let ser = serialize(&tx).unwrap();
        let deser = deserialize(&ser).unwrap();
        assert_eq!(tx, deser);
    }

    #[test]
    fn test_transaction_json_serialize() {
        let legacy_version_json = r#"{ "blockTime": 1726125580, "meta": { "computeUnitsConsumed": 150, "err": null, "fee": 5000, "innerInstructions": [], "loadedAddresses": { "readonly": [], "writable": [] }, "logMessages": [ "Program 11111111111111111111111111111111 invoke [1]", "Program 11111111111111111111111111111111 success" ], "postBalances": [ 19999934990, 12999985016, 1 ], "postTokenBalances": [], "preBalances": [ 19999939991, 12999985015, 1 ], "preTokenBalances": [], "rewards": [], "status": { "Ok": null } }, "slot": 325448256, "transaction": { "message": { "accountKeys": [ "EabqyjABpFwUGhw2t2HVPGavjD1uqGm6ciMPhBRrdTxh", "9ri4mUToddwCc6jg1GTL5sobkkFxjUzjZ6CZ6L91LzAR", "11111111111111111111111111111111" ], "header": { "numReadonlySignedAccounts": 0, "numReadonlyUnsignedAccounts": 1, "numRequiredSignatures": 1 }, "instructions": [ { "accounts": [ 0, 1 ], "data": "3Bxs412MvVNQj175", "programIdIndex": 2, "stackHeight": null } ], "recentBlockhash": "EMcudiFZWenakUVWtipQuu4ymZZcJmbsQFWUoPX4j35w" }, "signatures": [ "3t6afQP9Zp8FV49moN42x1QZCQYKHtpXYCakdpt1zxBHWQLbUHrhLCZmPxiNTN4A5HE6VJwnA2h5AjvZovqhcnGH" ] }, "version": "legacy" }"#;
        let numbered_version_json = r#"{ "blockTime": 1725954458, "meta": { "computeUnitsConsumed": 142936, "err": null, "fee": 7400, "innerInstructions": [ { "index": 1, "instructions": [ { "accounts": [ 3, 5, 19 ], "data": "3ay4cmSa9v5u", "programIdIndex": 20, "stackHeight": 2 } ] }, { "index": 2, "instructions": [ { "accounts": [ 9, 6, 22 ], "data": "3Dc4pim41vmd", "programIdIndex": 20, "stackHeight": 2 } ] }, { "index": 3, "instructions": [ { "accounts": [ 5, 10, 0 ], "data": "3KJnQat1Fiib", "programIdIndex": 20, "stackHeight": 2 } ] }, { "index": 4, "instructions": [ { "accounts": [ 6, 4, 0 ], "data": "3qottyySieFh", "programIdIndex": 20, "stackHeight": 2 } ] } ], "loadedAddresses": { "readonly": [], "writable": [] }, "logMessages": [ "Program ComputeBudget111111111111111111111111111111 invoke [1]", "Program ComputeBudget111111111111111111111111111111 success", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb invoke [1]", "Program log: Instruction: SettleFunds", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction: Transfer", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 782609 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program data: CjLwde1D5umUJll68E8UShvGApTsIHaTw8TYxQEDMHFw9zeM8g2hCYCXd/YBAAAAAAAAAAAAAAAAAAAAAAAAAAA=", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb consumed 23898 of 799850 compute units", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb success", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb invoke [1]", "Program log: Instruction: SettleFunds", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction: Transfer", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 758711 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program data: CjLwde1D5unNP4yqTqwfPi7MahPay9UVaVOsvuDYA+uA4QhOto5JcADgZzUAAAAAAAAAAAAAAAAAAAAAAAAAAAA=", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb consumed 23898 of 775952 compute units", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb success", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb invoke [1]", "Program log: Instruction: CancelAllAndPlaceOrders", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction: Transfer", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 712315 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb consumed 46738 of 752054 compute units", "Program return: opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb AgAAAAEVi/z//////1rMDgAAAAAAARSL/P//////hUEPAAAAAAA=", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb success", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb invoke [1]", "Program log: Instruction: CancelAllAndPlaceOrders", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction: Transfer", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 664063 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb consumed 48252 of 705316 compute units", "Program return: opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb AgAAAAFAavj//////1bNDgAAAAAAAT9q+P//////iEIPAAAAAAA=", "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb success" ], "postBalances": [ 588111184, 9688420, 6793060, 2039380, 2039380, 2039280, 2039280, 9688320, 6792960, 2039280, 2039280, 633916800, 633916800, 636255360, 633916900, 633916900, 636255460, 1, 1141440, 0, 934087680, 1, 0 ], "postTokenBalances": [ { "accountIndex": 3, "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "owner": "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "3718000000", "decimals": 6, "uiAmount": 3718.0, "uiAmountString": "3718" } }, { "accountIndex": 4, "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "owner": "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "19913589900", "decimals": 6, "uiAmount": 19913.5899, "uiAmountString": "19913.5899" } }, { "accountIndex": 5, "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "owner": "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "158303", "decimals": 6, "uiAmount": 0.158303, "uiAmountString": "0.158303" } }, { "accountIndex": 6, "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "owner": "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "235942", "decimals": 6, "uiAmount": 0.235942, "uiAmountString": "0.235942" } }, { "accountIndex": 9, "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "owner": "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "23000000", "decimals": 6, "uiAmount": 23.0, "uiAmountString": "23" } }, { "accountIndex": 10, "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "owner": "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "262364494446", "decimals": 6, "uiAmount": 262364.494446, "uiAmountString": "262364.494446" } } ], "preBalances": [ 588118584, 9688420, 6793060, 2039380, 2039380, 2039280, 2039280, 9688320, 6792960, 2039280, 2039280, 633916800, 633916800, 636255360, 633916900, 633916900, 636255460, 1, 1141440, 0, 934087680, 1, 0 ], "preTokenBalances": [ { "accountIndex": 3, "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "owner": "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "12148000000", "decimals": 6, "uiAmount": 12148.0, "uiAmountString": "12148" } }, { "accountIndex": 4, "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "owner": "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "19017494707", "decimals": 6, "uiAmount": 19017.494707, "uiAmountString": "19017.494707" } }, { "accountIndex": 5, "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "owner": "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "313857", "decimals": 6, "uiAmount": 0.313857, "uiAmountString": "0.313857" } }, { "accountIndex": 6, "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "owner": "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "331135", "decimals": 6, "uiAmount": 0.331135, "uiAmountString": "0.331135" } }, { "accountIndex": 9, "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "owner": "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "919000000", "decimals": 6, "uiAmount": 919.0, "uiAmountString": "919" } }, { "accountIndex": 10, "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "owner": "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i", "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "uiTokenAmount": { "amount": "253934338892", "decimals": 6, "uiAmount": 253934.338892, "uiAmountString": "253934.338892" } } ], "returnData": { "data": [ "AgAAAAFAavj//////1bNDgAAAAAAAT9q+P//////iEIPAAAAAAA=", "base64" ], "programId": "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb" }, "rewards": [], "status": { "Ok": null } }, "slot": 288918198, "transaction": { "message": { "accountKeys": [ "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx", "AyKFqUZd6FNt2jMeg7Bhbu4cS8RKMQxp2owBtjqVG1gU", "ECWHdhxPX2Vx299xvB1Dd5aTegGJmFcXWhrESftsKkRc", "8sxE2FYK3dzkXQ4oy8phVoJBZU9jgvpUuJWAhwDLtmfP", "Cb6wY5fpawswXhSS5hqCE3xW3Qw2GjA7soxjLNnarvCd", "458uFwc7urgyWwed7wRVLvY9h2eTxAKSeSA7M2f5hj5e", "eWmZj4TDx4tzgn49texEgDzABTzkrrQzgSiDpv3DSL5", "EpCnL2VMaSTgr7VPVaA2f69Y4iB5xAPivNAPgg7uHWab", "ArAsfvhAKb9F7H8SWhsZzf3ncRdzs7kMSaw1fxiyg9P2", "7sViAX3S7rzd3fbN56AsDEzehbgmi8We3NHBwbb7UHfU", "8uT1JEq9bxcXPNtgBJeXzQgDwojxkjxy1PC8uSEQuSBG", "BbZkRsLdtqJ8wsNkcGCkbJ3XN4ndR4PEFKGjyMWs4E9T", "5mgHN3wagFGSnPHN9VaDJp2BdbQonQ1bJSPow1Dcav3u", "5mGwZCeVcTjoWfs2uDgLEaP8H9F1GZwhPiKXucPyS5JY", "HGaofFyMA9xeePf7if3LtnhbciPuYcCKbqadNKka7MSq", "H1f8y3ujiHA6Uk9eBjhgmdCEq38rvqbWj2vjegLv5ryk", "9YacbvkSFuQ1nNtzshrARnM7HdPwidpc4MWySU9DMEKt", "ComputeBudget111111111111111111111111111111", "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb", "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU", "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "11111111111111111111111111111111", "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i" ], "addressTableLookups": [], "header": { "numReadonlySignedAccounts": 0, "numReadonlyUnsignedAccounts": 6, "numRequiredSignatures": 1 }, "instructions": [ { "accounts": [], "data": "3kF1YY3pexPH", "programIdIndex": 17, "stackHeight": null }, { "accounts": [ 0, 0, 1, 2, 19, 3, 4, 5, 6, 18, 20, 21 ], "data": "grMARW36kxQ", "programIdIndex": 18, "stackHeight": null }, { "accounts": [ 0, 0, 7, 8, 22, 9, 10, 6, 5, 18, 20, 21 ], "data": "grMARW36kxQ", "programIdIndex": 18, "stackHeight": null }, { "accounts": [ 0, 7, 18, 5, 6, 8, 11, 12, 13, 10, 9, 21, 21, 20 ], "data": "s6Pv7kdeSzJWnWia6ULkQwqtfTkJQb2daW2MXMaXTXMxfzqmYwZcxYTHhRQHMkLj3mDnVy3uzZt8VY7DVMfw5P9Hmu", "programIdIndex": 18, "stackHeight": null }, { "accounts": [ 0, 1, 18, 6, 5, 2, 14, 15, 16, 4, 3, 21, 21, 20 ], "data": "s6Pv7kdeSzJWnWia6UGWraJGzKaE5xs9pKLo8fMPtZn4uXTtWwJLmiTNPHVEiXNrWZjrCH5g6gxfZxwb7UBr2tUVHu", "programIdIndex": 18, "stackHeight": null } ], "recentBlockhash": "4Piqad6azGRvWcKS2xqv9ThjqGiSLrAycye5PztpMDYP" }, "signatures": [ "3kxL8Qvp16kmVNiUkQSJ3zLvCJDK4qPZZ1ZL8W2VHeYoJUJnQ4VqMHFMNSmsGBq7rTfpe8cTzCopMSNRen6vGFt1" ] }, "version": 0 }"#;

        let test_tx = EncodedConfirmedTransactionWithStatusMeta {
            slot: 325448256,
            block_time: Some(1726125580),
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec![],
                    message: UiMessage::Parsed(UiParsedMessage {
                        account_keys: vec![],
                        recent_blockhash: "".to_string(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),
                meta: None,
                version: Some(TransactionVersion::Legacy(Legacy::Legacy)),
            },
        };

        let test_json = serde_json::to_string(&test_tx).unwrap();

        let test_tx_2: EncodedConfirmedTransactionWithStatusMeta =
            serde_json::from_str(&test_json).unwrap();

        assert_eq!(test_tx, test_tx_2); // This test fails because TransactionVersion is not serialized as a string, but as a null

        let _: EncodedConfirmedTransactionWithStatusMeta =
            serde_json::from_str(legacy_version_json).unwrap();

        let _: EncodedConfirmedTransactionWithStatusMeta =
            serde_json::from_str(numbered_version_json).unwrap();

        let test_tx = EncodedConfirmedTransactionWithStatusMeta {
            slot: 325448256,
            block_time: Some(1726125580),
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Accounts(UiAccountsList {
                    signatures: vec![
                        "vAjYpEH66M59GMVqsrmZdymmXHn8SRhUvehrcpfcEnQaNPpQg1k9w22FhQNjLSfzSAQDG3uVzpN8wS1qRnLUH6S"
                            .to_string(),
                    ],
                    account_keys: vec![AccountKey {
                        pubkey: "".to_string(),
                        writable: false,
                        signer: false,
                        source: None,
                    }],
                }),
                meta: None,
                version: Some(TransactionVersion::Legacy(Legacy::Legacy)),
            },
        };

        let test_json = serde_json::to_string(&test_tx).unwrap();
        let _: EncodedConfirmedTransactionWithStatusMeta =
            serde_json::from_str(&test_json).unwrap();
    }
}

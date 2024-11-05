//! This module contains tagged types for the types in the `super` module.
//!
//! Default implementations work with JSON serialization and deserialization, but
//! the `CandidType` trait doesn't support flattened or untagged enums, so we have to copy and paste
//! the "tagged" versions of structs to send them as candid types.

use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::types::{
    CommitmentLevel, Epoch, Legacy, ParsedAccount, ParsedInstruction, Rewards, RpcBlockProductionRange, Slot,
    TransactionBinaryEncoding, TransactionError, TransactionResult, UiAccountEncoding, UiAccountsList,
    UiCompiledInstruction, UiLoadedAddresses, UiParsedMessage, UiPartiallyDecodedInstruction, UiRawMessage,
    UiTokenAmount, UiTransactionReturnData, UiTransactionTokenBalance, UnixTimestamp,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockProductionConfig {
    pub identity: Option<String>, // validator identity, as a base-58 encoded string
    pub range: Option<RpcBlockProductionRange>, // current epoch if `None`
    pub commitment: Option<CommitmentLevel>,
}

impl From<RpcBlockProductionConfig> for super::RpcBlockProductionConfig {
    fn from(value: RpcBlockProductionConfig) -> Self {
        super::RpcBlockProductionConfig {
            identity: value.identity,
            range: value.range,
            commitment: value.commitment,
        }
    }
}

impl From<super::RpcBlockProductionConfig> for RpcBlockProductionConfig {
    fn from(value: super::RpcBlockProductionConfig) -> Self {
        RpcBlockProductionConfig {
            identity: value.identity,
            range: value.range,
            commitment: value.commitment,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
pub struct RpcTokenAccountBalance {
    pub address: String,
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

impl From<RpcTokenAccountBalance> for super::RpcTokenAccountBalance {
    fn from(value: RpcTokenAccountBalance) -> Self {
        Self {
            address: value.address,
            amount: UiTokenAmount {
                amount: value.amount,
                decimals: value.decimals,
                ui_amount: value.ui_amount,
                ui_amount_string: value.ui_amount_string,
            },
        }
    }
}

impl From<super::RpcTokenAccountBalance> for RpcTokenAccountBalance {
    fn from(value: super::RpcTokenAccountBalance) -> Self {
        Self {
            address: value.address,
            amount: value.amount.amount,
            decimals: value.amount.decimals,
            ui_amount: value.amount.ui_amount,
            ui_amount_string: value.amount.ui_amount_string,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, CandidType)]
pub struct UiConfirmedBlock {
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: String,
    pub blockhash: String,
    #[serde(rename = "parentSlot")]
    pub parent_slot: Slot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<EncodedTransactionWithStatusMeta>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewards: Option<Rewards>,
    #[serde(default, rename = "numRewardPartitions", skip_serializing_if = "Option::is_none")]
    pub num_reward_partitions: Option<u64>,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
}

impl From<UiConfirmedBlock> for super::UiConfirmedBlock {
    fn from(value: UiConfirmedBlock) -> Self {
        Self {
            previous_blockhash: value.previous_blockhash,
            blockhash: value.blockhash,
            parent_slot: value.parent_slot,
            transactions: value
                .transactions
                .map(|transactions| transactions.into_iter().map(Into::into).collect()),
            signatures: value.signatures,
            rewards: value.rewards,
            num_reward_partitions: value.num_reward_partitions,
            block_time: value.block_time,
            block_height: value.block_height,
        }
    }
}

impl From<super::UiConfirmedBlock> for UiConfirmedBlock {
    fn from(value: super::UiConfirmedBlock) -> Self {
        Self {
            previous_blockhash: value.previous_blockhash,
            blockhash: value.blockhash,
            parent_slot: value.parent_slot,
            transactions: value
                .transactions
                .map(|transactions| transactions.into_iter().map(Into::into).collect()),
            signatures: value.signatures,
            rewards: value.rewards,
            num_reward_partitions: value.num_reward_partitions,
            block_time: value.block_time,
            block_height: value.block_height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub enum UiAccountData {
    #[serde(rename = "legacyBinary")]
    LegacyBinary(String), // Legacy. Retained for RPC backwards compatibility
    #[serde(rename = "json")]
    Json(ParsedAccount),
    #[serde(rename = "binary")]
    Binary(String, UiAccountEncoding),
}

impl From<super::UiAccountData> for UiAccountData {
    fn from(value: super::UiAccountData) -> Self {
        match value {
            super::UiAccountData::LegacyBinary(blob) => Self::LegacyBinary(blob),
            super::UiAccountData::Json(parsed) => Self::Json(parsed),
            super::UiAccountData::Binary(blob, encoding) => Self::Binary(blob, encoding),
        }
    }
}

impl From<UiAccountData> for super::UiAccountData {
    fn from(value: UiAccountData) -> Self {
        match value {
            UiAccountData::LegacyBinary(blob) => Self::LegacyBinary(blob),
            UiAccountData::Json(parsed) => Self::Json(parsed),
            UiAccountData::Binary(blob, encoding) => Self::Binary(blob, encoding),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, CandidType)]
pub struct UiAccount {
    pub lamports: u64,
    pub data: UiAccountData,
    pub owner: String,
    pub executable: bool,
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: Epoch,
    pub space: Option<u64>,
}

impl From<UiAccount> for super::UiAccount {
    fn from(value: UiAccount) -> Self {
        Self {
            lamports: value.lamports,
            data: value.data.into(),
            owner: value.owner,
            executable: value.executable,
            rent_epoch: value.rent_epoch,
            space: value.space,
        }
    }
}

impl From<super::UiAccount> for UiAccount {
    fn from(value: super::UiAccount) -> Self {
        Self {
            lamports: value.lamports,
            data: value.data.into(),
            owner: value.owner,
            executable: value.executable,
            rent_epoch: value.rent_epoch,
            space: value.space,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, CandidType)]
pub struct RpcKeyedAccount {
    pub pubkey: String,
    pub account: UiAccount,
}

impl From<RpcKeyedAccount> for super::RpcKeyedAccount {
    fn from(tagged: RpcKeyedAccount) -> Self {
        Self {
            pubkey: tagged.pubkey,
            account: tagged.account.into(),
        }
    }
}

impl From<super::RpcKeyedAccount> for RpcKeyedAccount {
    fn from(value: super::RpcKeyedAccount) -> Self {
        Self {
            pubkey: value.pubkey,
            account: value.account.into(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct EncodedConfirmedBlock {
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: String,
    pub blockhash: String,
    #[serde(rename = "parentSlot")]
    pub parent_slot: Slot,
    pub transactions: Vec<EncodedTransactionWithStatusMeta>,
    pub rewards: Rewards,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
}

impl From<super::EncodedConfirmedBlock> for EncodedConfirmedBlock {
    fn from(value: super::EncodedConfirmedBlock) -> Self {
        let super::EncodedConfirmedBlock {
            previous_blockhash,
            blockhash,
            parent_slot,
            transactions,
            rewards,
            block_time,
            block_height,
        } = value;
        Self {
            previous_blockhash,
            blockhash,
            parent_slot,
            transactions: transactions.into_iter().map(Into::into).collect(),
            rewards,
            block_time,
            block_height,
        }
    }
}

impl From<EncodedConfirmedBlock> for super::EncodedConfirmedBlock {
    fn from(value: EncodedConfirmedBlock) -> Self {
        let EncodedConfirmedBlock {
            previous_blockhash,
            blockhash,
            parent_slot,
            transactions,
            rewards,
            block_time,
            block_height,
        } = value;
        Self {
            previous_blockhash,
            blockhash,
            parent_slot,
            transactions: transactions.into_iter().map(Into::into).collect(),
            rewards,
            block_time,
            block_height,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct EncodedConfirmedTransactionWithStatusMeta {
    pub slot: Slot,
    pub transaction: EncodedTransactionWithStatusMeta,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
}

impl From<super::EncodedConfirmedTransactionWithStatusMeta> for EncodedConfirmedTransactionWithStatusMeta {
    fn from(value: super::EncodedConfirmedTransactionWithStatusMeta) -> Self {
        let super::EncodedConfirmedTransactionWithStatusMeta {
            slot,
            transaction,
            block_time,
        } = value;
        Self {
            slot,
            transaction: transaction.into(),
            block_time,
        }
    }
}

impl From<EncodedConfirmedTransactionWithStatusMeta> for super::EncodedConfirmedTransactionWithStatusMeta {
    fn from(value: EncodedConfirmedTransactionWithStatusMeta) -> Self {
        let EncodedConfirmedTransactionWithStatusMeta {
            slot,
            transaction,
            block_time,
        } = value;
        Self {
            slot,
            transaction: transaction.into(),
            block_time,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct EncodedTransactionWithStatusMeta {
    pub transaction: EncodedTransaction,
    pub meta: Option<UiTransactionStatusMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<TransactionVersion>,
}

impl From<super::EncodedTransactionWithStatusMeta> for EncodedTransactionWithStatusMeta {
    fn from(value: super::EncodedTransactionWithStatusMeta) -> Self {
        let super::EncodedTransactionWithStatusMeta {
            transaction,
            meta,
            version,
        } = value;
        Self {
            transaction: transaction.into(),
            meta: meta.map(Into::into),
            version: version.map(Into::into),
        }
    }
}

impl From<EncodedTransactionWithStatusMeta> for super::EncodedTransactionWithStatusMeta {
    fn from(value: EncodedTransactionWithStatusMeta) -> Self {
        let EncodedTransactionWithStatusMeta {
            transaction,
            meta,
            version,
        } = value;
        Self {
            transaction: transaction.into(),
            meta: meta.map(Into::into),
            version: version.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum TransactionVersion {
    #[serde(rename = "legacy")]
    Legacy(Legacy),
    #[serde(rename = "number")]
    Number(u8),
}

impl From<super::TransactionVersion> for TransactionVersion {
    fn from(value: super::TransactionVersion) -> Self {
        match value {
            super::TransactionVersion::Legacy(_) => Self::Legacy(Legacy::Legacy),
            super::TransactionVersion::Number(version) => Self::Number(version),
        }
    }
}

impl From<TransactionVersion> for super::TransactionVersion {
    fn from(value: TransactionVersion) -> Self {
        match value {
            TransactionVersion::Legacy(_) => Self::Legacy(Legacy::Legacy),
            TransactionVersion::Number(version) => Self::Number(version),
        }
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "innerInstructions")]
    pub inner_instructions: Option<Vec<UiInnerInstructions>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "logMessages")]
    pub log_messages: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "preTokenBalances")]
    pub pre_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "postTokenBalances")]
    pub post_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewards: Option<Rewards>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "loadedAddresses")]
    pub loaded_addresses: Option<UiLoadedAddresses>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "returnData")]
    pub return_data: Option<UiTransactionReturnData>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "computeUnitsConsumed")]
    pub compute_units_consumed: Option<u64>,
}

impl From<super::UiTransactionStatusMeta> for UiTransactionStatusMeta {
    fn from(value: super::UiTransactionStatusMeta) -> Self {
        let super::UiTransactionStatusMeta {
            err,
            status,
            fee,
            pre_balances,
            post_balances,
            inner_instructions,
            log_messages,
            pre_token_balances,
            post_token_balances,
            rewards,
            loaded_addresses,
            return_data,
            compute_units_consumed,
        } = value;
        Self {
            err,
            status,
            fee,
            pre_balances,
            post_balances,
            inner_instructions: inner_instructions
                .map(|inner_instructions| inner_instructions.into_iter().map(Into::into).collect()),
            log_messages,
            pre_token_balances,
            post_token_balances,
            rewards,
            loaded_addresses,
            return_data,
            compute_units_consumed,
        }
    }
}

impl From<UiTransactionStatusMeta> for super::UiTransactionStatusMeta {
    fn from(value: UiTransactionStatusMeta) -> Self {
        let UiTransactionStatusMeta {
            err,
            status,
            fee,
            pre_balances,
            post_balances,
            inner_instructions,
            log_messages,
            pre_token_balances,
            post_token_balances,
            rewards,
            loaded_addresses,
            return_data,
            compute_units_consumed,
        } = value;
        Self {
            err,
            status,
            fee,
            pre_balances,
            post_balances,
            inner_instructions: inner_instructions
                .map(|inner_instructions| inner_instructions.into_iter().map(Into::into).collect()),
            log_messages,
            pre_token_balances,
            post_token_balances,
            rewards,
            loaded_addresses,
            return_data,
            compute_units_consumed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum EncodedTransaction {
    #[serde(rename = "legacyBinary")]
    LegacyBinary(String), /* Old way of expressing base-58, retained for RPC backwards
                           * compatibility */
    #[serde(rename = "binary")]
    Binary(String, TransactionBinaryEncoding),
    #[serde(rename = "json")]
    Json(UiTransaction),
    #[serde(rename = "accounts")]
    Accounts(UiAccountsList),
}

impl From<super::EncodedTransaction> for EncodedTransaction {
    fn from(value: super::EncodedTransaction) -> Self {
        match value {
            super::EncodedTransaction::LegacyBinary(blob) => Self::LegacyBinary(blob),
            super::EncodedTransaction::Binary(blob, encoding) => Self::Binary(blob, encoding),
            super::EncodedTransaction::Json(tx) => Self::Json(tx.into()),
            super::EncodedTransaction::Accounts(acc_list) => Self::Accounts(acc_list),
        }
    }
}

impl From<EncodedTransaction> for super::EncodedTransaction {
    fn from(value: EncodedTransaction) -> Self {
        match value {
            EncodedTransaction::LegacyBinary(blob) => Self::LegacyBinary(blob),
            EncodedTransaction::Binary(blob, encoding) => Self::Binary(blob, encoding),
            EncodedTransaction::Json(ui_transaction) => Self::Json(ui_transaction.into()),
            EncodedTransaction::Accounts(ui_accounts_list) => Self::Accounts(ui_accounts_list),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct UiTransaction {
    pub signatures: Vec<String>,
    pub message: UiMessage,
}

impl From<super::UiTransaction> for UiTransaction {
    fn from(value: super::UiTransaction) -> Self {
        let super::UiTransaction { signatures, message } = value;
        Self {
            signatures,
            message: message.into(),
        }
    }
}

impl From<UiTransaction> for super::UiTransaction {
    fn from(value: UiTransaction) -> Self {
        let UiTransaction { signatures, message } = value;
        Self {
            signatures,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum UiMessage {
    #[serde(rename = "parsed")]
    Parsed(UiParsedMessage),
    #[serde(rename = "raw")]
    Raw(UiRawMessage),
}

impl From<super::UiMessage> for UiMessage {
    fn from(value: super::UiMessage) -> Self {
        match value {
            super::UiMessage::Parsed(parsed) => Self::Parsed(parsed),
            super::UiMessage::Raw(raw) => Self::Raw(raw),
        }
    }
}

impl From<UiMessage> for super::UiMessage {
    fn from(value: UiMessage) -> Self {
        match value {
            UiMessage::Parsed(parsed) => Self::Parsed(parsed),
            UiMessage::Raw(raw) => Self::Raw(raw),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct UiInnerInstructions {
    /// Transaction instruction index
    pub index: u8,
    /// List of inner instructions
    pub instructions: Vec<UiInstruction>,
}

impl From<UiInnerInstructions> for super::UiInnerInstructions {
    fn from(value: UiInnerInstructions) -> Self {
        Self {
            index: value.index,
            instructions: value.instructions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<super::UiInnerInstructions> for UiInnerInstructions {
    fn from(value: super::UiInnerInstructions) -> Self {
        Self {
            index: value.index,
            instructions: value.instructions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum UiInstruction {
    #[serde(rename = "compiled")]
    Compiled(UiCompiledInstruction),
    #[serde(rename = "parsed")]
    Parsed(UiParsedInstruction),
}

impl From<UiInstruction> for super::UiInstruction {
    fn from(value: UiInstruction) -> Self {
        match value {
            UiInstruction::Compiled(compiled) => Self::Compiled(compiled),
            UiInstruction::Parsed(parsed) => Self::Parsed(parsed.into()),
        }
    }
}

impl From<super::UiInstruction> for UiInstruction {
    fn from(value: super::UiInstruction) -> Self {
        match value {
            super::UiInstruction::Compiled(compiled) => Self::Compiled(compiled),
            super::UiInstruction::Parsed(parsed) => Self::Parsed(parsed.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub enum UiParsedInstruction {
    #[serde(rename = "parsed")]
    Parsed(ParsedInstruction),
    #[serde(rename = "partiallyDecoded")]
    PartiallyDecoded(UiPartiallyDecodedInstruction),
}

impl From<UiParsedInstruction> for super::UiParsedInstruction {
    fn from(value: UiParsedInstruction) -> Self {
        match value {
            UiParsedInstruction::Parsed(parsed) => Self::Parsed(parsed),
            UiParsedInstruction::PartiallyDecoded(partially_decoded) => Self::PartiallyDecoded(partially_decoded),
        }
    }
}

impl From<super::UiParsedInstruction> for UiParsedInstruction {
    fn from(value: super::UiParsedInstruction) -> Self {
        match value {
            super::UiParsedInstruction::Parsed(parsed) => Self::Parsed(parsed),
            super::UiParsedInstruction::PartiallyDecoded(partially_decoded) => {
                Self::PartiallyDecoded(partially_decoded)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct RpcSimulateTransactionResult {
    pub err: Option<TransactionError>,
    pub logs: Option<Vec<String>>,
    pub accounts: Option<Vec<Option<UiAccount>>>,
    #[serde(rename = "unitsConsumed")]
    pub units_consumed: Option<u64>,
    #[serde(rename = "returnData")]
    pub return_data: Option<UiTransactionReturnData>,
    #[serde(rename = "innerInstructions")]
    pub inner_instructions: Option<Vec<UiInnerInstructions>>,
}

impl From<RpcSimulateTransactionResult> for super::RpcSimulateTransactionResult {
    fn from(value: RpcSimulateTransactionResult) -> Self {
        Self {
            err: value.err,
            logs: value.logs,
            accounts: value
                .accounts
                .map(|accounts| accounts.into_iter().map(|acc| acc.map(Into::into)).collect()),
            units_consumed: value.units_consumed,
            return_data: value.return_data,
            inner_instructions: value
                .inner_instructions
                .map(|ixs| ixs.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<super::RpcSimulateTransactionResult> for RpcSimulateTransactionResult {
    fn from(value: super::RpcSimulateTransactionResult) -> Self {
        Self {
            err: value.err,
            logs: value.logs,
            accounts: value
                .accounts
                .map(|accounts| accounts.into_iter().map(|acc| acc.map(Into::into)).collect()),
            units_consumed: value.units_consumed,
            return_data: value.return_data,
            inner_instructions: value
                .inner_instructions
                .map(|ixs| ixs.into_iter().map(Into::into).collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use candid::{Decode, Encode};

    use super::*;
    use crate::{rpc_client::JsonRpcResponse, types::OptionalContext};

    #[test]
    fn test_transaction_serialize() {
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

        let encoded = Encode!(&test_tx).unwrap();
        let decoded = Decode!(&encoded, EncodedConfirmedTransactionWithStatusMeta).unwrap();

        assert_eq!(test_tx, decoded);
    }

    #[test]
    fn test_confirmed_block_serialize() {
        let json = r#"{"blockHeight":278596380,"blockTime":1712339598,"blockhash":"DuDqV3wED1kmx8RarqTmzGbwjCdBCyswMu6vSuAJGmhQ","parentSlot":290304299,"previousBlockhash":"2MssJkEXpFZthuFFUMuy9ZKtPXvg4TnUWog1E3vq2QPd","signatures":["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"]}"#;
        let block = serde_json::from_str::<UiConfirmedBlock>(json).unwrap();
        let encoded = Encode!(&block).unwrap();
        let decoded = Decode!(&encoded, UiConfirmedBlock).unwrap();
        assert_eq!(block, decoded);
    }

    #[test]
    fn test_rpc_keyed_account_serialize() {
        let json = r#"{"jsonrpc":"2.0","result":{"context":{"slot":298362228},
        "value":[{
            "account":{
                "data":{
                    "program":"spl-token",
                    "parsed":{"info":{
                        "tokenAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},
                        "delegate":"4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T",
                        "delegatedAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},
                        "state":"initialized",
                        "isNative":false,
                        "mint":"3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E",
                        "owner":"CnPoSPKXu7wJqxe59Fs72tkBeALovhsCxYeFwPCQH9TD"},
                        "type":"account"
                    },
                    "space":165
                },
                "executable":false,
                "lamports":1726080,
                "owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "rentEpoch":4,
                "space":165
            },
            "pubkey":"28YTZEwqtMHWrhWcvv34se7pjS7wctgqzCPB3gReCFKp"
        }]}, "id":1}"#;
        let res =
            serde_json::from_str::<JsonRpcResponse<OptionalContext<Vec<super::super::RpcKeyedAccount>>>>(json).unwrap();
        assert!(res.result.is_some())
    }
}

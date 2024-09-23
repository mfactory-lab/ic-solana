//! This module contains tagged types for the types in the `super` module.
//! Default implementations work with json serialization and deserialization, but
//! the `CandidType` trait doesn't support flattened or untagged enums, so we have to copy paste
//! the "tagged" versions of structs in order to send them as candid types.
use {
    super::{
        CommitmentConfig, EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
        EncodedTransactionWithStatusMeta, Epoch, Legacy, ParsedAccount, ParsedInstruction, Rewards,
        RpcBlockProductionConfig, Slot, TransactionBinaryEncoding, TransactionError,
        TransactionResult, TransactionVersion, UiAccount, UiAccountData, UiAccountEncoding,
        UiAccountsList, UiCompiledInstruction, UiConfirmedBlock, UiInnerInstructions,
        UiInstruction, UiLoadedAddresses, UiMessage, UiParsedInstruction, UiParsedMessage,
        UiPartiallyDecodedInstruction, UiRawMessage, UiTokenAmount, UiTransaction,
        UiTransactionReturnData, UiTransactionStatusMeta, UiTransactionTokenBalance, UnixTimestamp,
    },
    crate::response::{
        EncodedConfirmedBlock, RpcBlockProductionRange, RpcKeyedAccount, RpcTokenAccountBalance,
    },
    candid::CandidType,
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, CandidType, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaggedRpcBlockProductionConfig {
    pub identity: Option<String>, // validator identity, as a base-58 encoded string
    pub range: Option<RpcBlockProductionRange>, // current epoch if `None`
    pub commitment: Option<CommitmentConfig>,
}

impl From<TaggedRpcBlockProductionConfig> for RpcBlockProductionConfig {
    fn from(tagged: TaggedRpcBlockProductionConfig) -> Self {
        RpcBlockProductionConfig {
            identity: tagged.identity,
            range: tagged.range,
            commitment: tagged.commitment,
        }
    }
}

impl From<RpcBlockProductionConfig> for TaggedRpcBlockProductionConfig {
    fn from(rpc_block_production_config: RpcBlockProductionConfig) -> Self {
        TaggedRpcBlockProductionConfig {
            identity: rpc_block_production_config.identity,
            range: rpc_block_production_config.range,
            commitment: rpc_block_production_config.commitment,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, CandidType)]
pub struct TaggedRpcTokenAccountBalance {
    pub address: String,
    pub amount: UiTokenAmount,
}

impl From<TaggedRpcTokenAccountBalance> for RpcTokenAccountBalance {
    fn from(tagged: TaggedRpcTokenAccountBalance) -> Self {
        RpcTokenAccountBalance {
            address: tagged.address,
            amount: tagged.amount,
        }
    }
}

impl From<RpcTokenAccountBalance> for TaggedRpcTokenAccountBalance {
    fn from(rpc_token_account_balance: RpcTokenAccountBalance) -> Self {
        TaggedRpcTokenAccountBalance {
            address: rpc_token_account_balance.address,
            amount: rpc_token_account_balance.amount,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, CandidType)]
pub struct TaggedUiConfirmedBlock {
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: String,
    pub blockhash: String,
    #[serde(rename = "parentSlot")]
    pub parent_slot: Slot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<TaggedEncodedTransactionWithStatusMeta>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewards: Option<Rewards>,
    #[serde(
        default,
        rename = "numRewardPartitions",
        skip_serializing_if = "Option::is_none"
    )]
    pub num_reward_partitions: Option<u64>,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
}

impl From<TaggedUiConfirmedBlock> for UiConfirmedBlock {
    fn from(tagged: TaggedUiConfirmedBlock) -> Self {
        UiConfirmedBlock {
            previous_blockhash: tagged.previous_blockhash,
            blockhash: tagged.blockhash,
            parent_slot: tagged.parent_slot,
            transactions: tagged
                .transactions
                .map(|transactions| transactions.into_iter().map(Into::into).collect()),
            signatures: tagged.signatures,
            rewards: tagged.rewards,
            num_reward_partitions: tagged.num_reward_partitions,
            block_time: tagged.block_time,
            block_height: tagged.block_height,
        }
    }
}

impl From<UiConfirmedBlock> for TaggedUiConfirmedBlock {
    fn from(ui_confirmed_block: UiConfirmedBlock) -> Self {
        TaggedUiConfirmedBlock {
            previous_blockhash: ui_confirmed_block.previous_blockhash,
            blockhash: ui_confirmed_block.blockhash,
            parent_slot: ui_confirmed_block.parent_slot,
            transactions: ui_confirmed_block
                .transactions
                .map(|transactions| transactions.into_iter().map(Into::into).collect()),
            signatures: ui_confirmed_block.signatures,
            rewards: ui_confirmed_block.rewards,
            num_reward_partitions: ui_confirmed_block.num_reward_partitions,
            block_time: ui_confirmed_block.block_time,
            block_height: ui_confirmed_block.block_height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub enum TaggedUiAccountData {
    #[serde(rename = "legacyBinary")]
    LegacyBinary(String), // Legacy. Retained for RPC backwards compatibility
    #[serde(rename = "json")]
    Json(ParsedAccount),
    #[serde(rename = "binary")]
    Binary(String, UiAccountEncoding),
}

impl From<UiAccountData> for TaggedUiAccountData {
    fn from(ui_account_data: UiAccountData) -> Self {
        match ui_account_data {
            UiAccountData::LegacyBinary(blob) => TaggedUiAccountData::LegacyBinary(blob),
            UiAccountData::Json(parsed) => TaggedUiAccountData::Json(parsed),
            UiAccountData::Binary(blob, encoding) => TaggedUiAccountData::Binary(blob, encoding),
        }
    }
}

impl From<TaggedUiAccountData> for UiAccountData {
    fn from(tagged_ui_account_data: TaggedUiAccountData) -> Self {
        match tagged_ui_account_data {
            TaggedUiAccountData::LegacyBinary(blob) => UiAccountData::LegacyBinary(blob),
            TaggedUiAccountData::Json(parsed) => UiAccountData::Json(parsed),
            TaggedUiAccountData::Binary(blob, encoding) => UiAccountData::Binary(blob, encoding),
        }
    }
}

/// A duplicate representation of an Account for pretty JSON serialization
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, CandidType)]
pub struct TaggedUiAccount {
    pub lamports: u64,
    pub data: TaggedUiAccountData,
    pub owner: String,
    pub executable: bool,
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: Epoch,
    pub space: Option<u64>,
}

impl From<TaggedUiAccount> for UiAccount {
    fn from(tagged: TaggedUiAccount) -> Self {
        UiAccount {
            lamports: tagged.lamports,
            data: tagged.data.into(),
            owner: tagged.owner,
            executable: tagged.executable,
            rent_epoch: tagged.rent_epoch,
            space: tagged.space,
        }
    }
}

impl From<UiAccount> for TaggedUiAccount {
    fn from(ui_account: UiAccount) -> Self {
        TaggedUiAccount {
            lamports: ui_account.lamports,
            data: ui_account.data.into(),
            owner: ui_account.owner,
            executable: ui_account.executable,
            rent_epoch: ui_account.rent_epoch,
            space: ui_account.space,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, CandidType)]
pub struct TaggedRpcKeyedAccount {
    pub pubkey: String,
    pub account: TaggedUiAccount,
}

impl From<TaggedRpcKeyedAccount> for RpcKeyedAccount {
    fn from(tagged: TaggedRpcKeyedAccount) -> Self {
        RpcKeyedAccount {
            pubkey: tagged.pubkey,
            account: tagged.account.into(),
        }
    }
}

impl From<RpcKeyedAccount> for TaggedRpcKeyedAccount {
    fn from(rpc_keyed_account: RpcKeyedAccount) -> Self {
        TaggedRpcKeyedAccount {
            pubkey: rpc_keyed_account.pubkey,
            account: rpc_keyed_account.account.into(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct TaggedEncodedConfirmedBlock {
    #[serde(rename = "previousBlockhash")]
    pub previous_blockhash: String,
    pub blockhash: String,
    #[serde(rename = "parentSlot")]
    pub parent_slot: Slot,
    pub transactions: Vec<TaggedEncodedTransactionWithStatusMeta>,
    pub rewards: Rewards,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
}

impl From<EncodedConfirmedBlock> for TaggedEncodedConfirmedBlock {
    fn from(encoded_confirmed_block: EncodedConfirmedBlock) -> Self {
        let EncodedConfirmedBlock {
            previous_blockhash,
            blockhash,
            parent_slot,
            transactions,
            rewards,
            block_time,
            block_height,
        } = encoded_confirmed_block;
        TaggedEncodedConfirmedBlock {
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

impl From<TaggedEncodedConfirmedBlock> for EncodedConfirmedBlock {
    fn from(tagged_encoded_confirmed_block: TaggedEncodedConfirmedBlock) -> Self {
        let TaggedEncodedConfirmedBlock {
            previous_blockhash,
            blockhash,
            parent_slot,
            transactions,
            rewards,
            block_time,
            block_height,
        } = tagged_encoded_confirmed_block;
        EncodedConfirmedBlock {
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
/// Tagged version of EncodedConfirmedTransactionWithStatusMeta
pub struct TaggedEncodedConfirmedTransactionWithStatusMeta {
    pub slot: Slot,
    pub transaction: TaggedEncodedTransactionWithStatusMeta,
    #[serde(rename = "blockTime")]
    pub block_time: Option<UnixTimestamp>,
}

impl From<EncodedConfirmedTransactionWithStatusMeta>
    for TaggedEncodedConfirmedTransactionWithStatusMeta
{
    fn from(encoded_confirmed_transaction: EncodedConfirmedTransactionWithStatusMeta) -> Self {
        let EncodedConfirmedTransactionWithStatusMeta {
            slot,
            transaction,
            block_time,
        } = encoded_confirmed_transaction;
        TaggedEncodedConfirmedTransactionWithStatusMeta {
            slot,
            transaction: transaction.into(),
            block_time,
        }
    }
}

impl From<TaggedEncodedConfirmedTransactionWithStatusMeta>
    for EncodedConfirmedTransactionWithStatusMeta
{
    fn from(
        tagged_encoded_confirmed_transaction: TaggedEncodedConfirmedTransactionWithStatusMeta,
    ) -> Self {
        let TaggedEncodedConfirmedTransactionWithStatusMeta {
            slot,
            transaction,
            block_time,
        } = tagged_encoded_confirmed_transaction;
        EncodedConfirmedTransactionWithStatusMeta {
            slot,
            transaction: transaction.into(),
            block_time,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
/// Tagged version of EncodedTransactionWithStatusMeta
pub struct TaggedEncodedTransactionWithStatusMeta {
    pub transaction: TaggedEncodedTransaction,
    pub meta: Option<TaggedUiTransactionStatusMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<TaggedTransactionVersion>,
}

impl From<EncodedTransactionWithStatusMeta> for TaggedEncodedTransactionWithStatusMeta {
    fn from(encoded_transaction: EncodedTransactionWithStatusMeta) -> Self {
        let EncodedTransactionWithStatusMeta {
            transaction,
            meta,
            version,
        } = encoded_transaction;
        TaggedEncodedTransactionWithStatusMeta {
            transaction: transaction.into(),
            meta: meta.map(Into::into),
            version: version.map(Into::into),
        }
    }
}

impl From<TaggedEncodedTransactionWithStatusMeta> for EncodedTransactionWithStatusMeta {
    fn from(tagged_encoded_transaction: TaggedEncodedTransactionWithStatusMeta) -> Self {
        let TaggedEncodedTransactionWithStatusMeta {
            transaction,
            meta,
            version,
        } = tagged_encoded_transaction;
        EncodedTransactionWithStatusMeta {
            transaction: transaction.into(),
            meta: meta.map(Into::into),
            version: version.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
/// Tagged version of TransactionVersion
pub enum TaggedTransactionVersion {
    #[serde(rename = "legacy")]
    Legacy(Legacy),
    #[serde(rename = "number")]
    Number(u8),
}

impl From<TransactionVersion> for TaggedTransactionVersion {
    fn from(transaction_version: TransactionVersion) -> Self {
        match transaction_version {
            TransactionVersion::Legacy(_) => TaggedTransactionVersion::Legacy(Legacy::Legacy),
            TransactionVersion::Number(version) => TaggedTransactionVersion::Number(version),
        }
    }
}

impl From<TaggedTransactionVersion> for TransactionVersion {
    fn from(tagged_transaction_version: TaggedTransactionVersion) -> Self {
        match tagged_transaction_version {
            TaggedTransactionVersion::Legacy(_) => TransactionVersion::Legacy(Legacy::Legacy),
            TaggedTransactionVersion::Number(version) => TransactionVersion::Number(version),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
/// Tagged version of EncodedTransaction
pub struct TaggedUiTransactionStatusMeta {
    pub err: Option<TransactionError>,
    pub status: TransactionResult<()>, // This field is deprecated.  See https://github.com/solana-labs/solana/issues/9302
    pub fee: u64,
    #[serde(rename = "preBalances")]
    pub pre_balances: Vec<u64>,
    #[serde(rename = "postBalances")]
    pub post_balances: Vec<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "innerInstructions"
    )]
    pub inner_instructions: Option<Vec<TaggedUiInnerInstructions>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "logMessages"
    )]
    pub log_messages: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "preTokenBalances"
    )]
    pub pre_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "postTokenBalances"
    )]
    pub post_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewards: Option<Rewards>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "loadedAddresses"
    )]
    pub loaded_addresses: Option<UiLoadedAddresses>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "returnData"
    )]
    pub return_data: Option<UiTransactionReturnData>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "computeUnitsConsumed"
    )]
    pub compute_units_consumed: Option<u64>,
}

impl From<UiTransactionStatusMeta> for TaggedUiTransactionStatusMeta {
    fn from(ui_transaction_status_meta: UiTransactionStatusMeta) -> Self {
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
        } = ui_transaction_status_meta;
        TaggedUiTransactionStatusMeta {
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

impl From<TaggedUiTransactionStatusMeta> for UiTransactionStatusMeta {
    fn from(tagged_ui_transaction_status_meta: TaggedUiTransactionStatusMeta) -> Self {
        let TaggedUiTransactionStatusMeta {
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
        } = tagged_ui_transaction_status_meta;
        UiTransactionStatusMeta {
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
/// Tagged version of EncodedTransaction
pub enum TaggedEncodedTransaction {
    #[serde(rename = "legacyBinary")]
    LegacyBinary(String), // Old way of expressing base-58, retained for RPC backwards compatibility
    #[serde(rename = "binary")]
    Binary(String, TransactionBinaryEncoding),
    #[serde(rename = "json")]
    Json(TaggedUiTransaction),
    #[serde(rename = "accounts")]
    Accounts(UiAccountsList),
}

impl From<EncodedTransaction> for TaggedEncodedTransaction {
    fn from(encoded_transaction: EncodedTransaction) -> Self {
        match encoded_transaction {
            EncodedTransaction::LegacyBinary(blob) => TaggedEncodedTransaction::LegacyBinary(blob),
            EncodedTransaction::Binary(blob, encoding) => {
                TaggedEncodedTransaction::Binary(blob, encoding)
            }
            EncodedTransaction::Json(ui_transaction) => {
                TaggedEncodedTransaction::Json(ui_transaction.into())
            }
            EncodedTransaction::Accounts(ui_accounts_list) => {
                TaggedEncodedTransaction::Accounts(ui_accounts_list)
            }
        }
    }
}

impl From<TaggedEncodedTransaction> for EncodedTransaction {
    fn from(tagged_encoded_transaction: TaggedEncodedTransaction) -> Self {
        match tagged_encoded_transaction {
            TaggedEncodedTransaction::LegacyBinary(blob) => EncodedTransaction::LegacyBinary(blob),
            TaggedEncodedTransaction::Binary(blob, encoding) => {
                EncodedTransaction::Binary(blob, encoding)
            }
            TaggedEncodedTransaction::Json(ui_transaction) => {
                EncodedTransaction::Json(ui_transaction.into())
            }
            TaggedEncodedTransaction::Accounts(ui_accounts_list) => {
                EncodedTransaction::Accounts(ui_accounts_list)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
/// Tagged version of UiTransaction
pub struct TaggedUiTransaction {
    pub signatures: Vec<String>,
    pub message: TaggedUiMessage,
}

impl From<UiTransaction> for TaggedUiTransaction {
    fn from(ui_transaction: UiTransaction) -> Self {
        let UiTransaction {
            signatures,
            message,
        } = ui_transaction;
        TaggedUiTransaction {
            signatures,
            message: message.into(),
        }
    }
}

impl From<TaggedUiTransaction> for UiTransaction {
    fn from(tagged_ui_transaction: TaggedUiTransaction) -> Self {
        let TaggedUiTransaction {
            signatures,
            message,
        } = tagged_ui_transaction;
        UiTransaction {
            signatures,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
/// Tagged version of UiMessage
pub enum TaggedUiMessage {
    #[serde(rename = "parsed")]
    Parsed(UiParsedMessage),
    #[serde(rename = "raw")]
    Raw(UiRawMessage),
}

impl From<UiMessage> for TaggedUiMessage {
    fn from(ui_message: UiMessage) -> Self {
        match ui_message {
            UiMessage::Parsed(parsed) => TaggedUiMessage::Parsed(parsed),
            UiMessage::Raw(raw) => TaggedUiMessage::Raw(raw),
        }
    }
}

impl From<TaggedUiMessage> for UiMessage {
    fn from(tagged: TaggedUiMessage) -> Self {
        match tagged {
            TaggedUiMessage::Parsed(parsed) => UiMessage::Parsed(parsed),
            TaggedUiMessage::Raw(raw) => UiMessage::Raw(raw),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
/// Tagged version of UiInnerInstructions
pub struct TaggedUiInnerInstructions {
    /// Transaction instruction index
    pub index: u8,
    /// List of inner instructions
    pub instructions: Vec<TaggedUiInstruction>,
}

impl From<TaggedUiInnerInstructions> for UiInnerInstructions {
    fn from(tagged: TaggedUiInnerInstructions) -> Self {
        Self {
            index: tagged.index,
            instructions: tagged.instructions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<UiInnerInstructions> for TaggedUiInnerInstructions {
    fn from(ui_inner_instructions: UiInnerInstructions) -> Self {
        Self {
            index: ui_inner_instructions.index,
            instructions: ui_inner_instructions
                .instructions
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
/// Tagged version of UiInstruction
pub enum TaggedUiInstruction {
    #[serde(rename = "compiled")]
    Compiled(UiCompiledInstruction),
    #[serde(rename = "parsed")]
    Parsed(TaggedUiParsedInstruction),
}

impl From<TaggedUiInstruction> for UiInstruction {
    fn from(tagged: TaggedUiInstruction) -> Self {
        match tagged {
            TaggedUiInstruction::Compiled(compiled) => UiInstruction::Compiled(compiled),
            TaggedUiInstruction::Parsed(parsed) => UiInstruction::Parsed(parsed.into()),
        }
    }
}

impl From<UiInstruction> for TaggedUiInstruction {
    fn from(ui_instruction: UiInstruction) -> Self {
        match ui_instruction {
            UiInstruction::Compiled(compiled) => TaggedUiInstruction::Compiled(compiled),
            UiInstruction::Parsed(parsed) => TaggedUiInstruction::Parsed(parsed.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
/// Tagged version of UiParsedInstruction
pub enum TaggedUiParsedInstruction {
    #[serde(rename = "parsed")]
    Parsed(ParsedInstruction),
    #[serde(rename = "partiallyDecoded")]
    PartiallyDecoded(UiPartiallyDecodedInstruction),
}

impl From<TaggedUiParsedInstruction> for UiParsedInstruction {
    fn from(tagged: TaggedUiParsedInstruction) -> Self {
        match tagged {
            TaggedUiParsedInstruction::Parsed(parsed) => UiParsedInstruction::Parsed(parsed),
            TaggedUiParsedInstruction::PartiallyDecoded(partially_decoded) => {
                UiParsedInstruction::PartiallyDecoded(partially_decoded)
            }
        }
    }
}

impl From<UiParsedInstruction> for TaggedUiParsedInstruction {
    fn from(ui_parsed_instruction: UiParsedInstruction) -> Self {
        match ui_parsed_instruction {
            UiParsedInstruction::Parsed(parsed) => TaggedUiParsedInstruction::Parsed(parsed),
            UiParsedInstruction::PartiallyDecoded(partially_decoded) => {
                TaggedUiParsedInstruction::PartiallyDecoded(partially_decoded)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        candid::{Decode, Encode},
    };

    #[test]
    fn test_transaction_candid_serialize() {
        let test_tx = TaggedEncodedConfirmedTransactionWithStatusMeta {
            slot: 325448256,
            block_time: Some(1726125580),
            transaction: TaggedEncodedTransactionWithStatusMeta {
                transaction: TaggedEncodedTransaction::Json(TaggedUiTransaction {
                    signatures: vec![],
                    message: TaggedUiMessage::Parsed(UiParsedMessage {
                        account_keys: vec![],
                        recent_blockhash: "".to_string(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),
                meta: None,
                version: Some(TaggedTransactionVersion::Legacy(Legacy::Legacy)),
            },
        };

        let encoded = Encode!(&test_tx).unwrap();
        let decoded = Decode!(&encoded, TaggedEncodedConfirmedTransactionWithStatusMeta).unwrap();

        assert_eq!(test_tx, decoded);
    }
}

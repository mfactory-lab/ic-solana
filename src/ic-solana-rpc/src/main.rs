use std::{collections::HashMap, str::FromStr};

use candid::{candid_method, Principal};
use ic_canisters_http_types::{
    HttpRequest as AssetHttpRequest, HttpResponse as AssetHttpResponse, HttpResponseBuilder,
};
use ic_cdk::{
    api::management_canister::http_request::{HttpResponse, TransformArgs},
    query, update,
};
use ic_solana::{
    metrics::{encode_metrics, read_metrics, Metrics},
    request::RpcRequest,
    rpc_client::{RpcConfig, RpcResult, RpcServices},
    types::{
        response::{
            RpcAccountBalance, RpcBlockCommitment, RpcBlockProduction, RpcBlockhash,
            RpcConfirmedTransactionStatusWithSignature, RpcContactInfo, RpcIdentity, RpcInflationGovernor,
            RpcInflationRate, RpcInflationReward, RpcLeaderSchedule, RpcPerfSample, RpcPrioritizationFee,
            RpcSnapshotSlotInfo, RpcSupply, RpcVersionInfo, RpcVoteAccountStatus,
        },
        tagged::{
            EncodedConfirmedTransactionWithStatusMeta, RpcBlockProductionConfig, RpcKeyedAccount,
            RpcSimulateTransactionResult, RpcTokenAccountBalance, UiAccount, UiConfirmedBlock,
        },
        CandidValue, CommitmentConfig, CommitmentLevel, EpochInfo, EpochSchedule, RpcAccountInfoConfig, RpcBlockConfig,
        RpcContextConfig, RpcEpochConfig, RpcGetVoteAccountsConfig, RpcLargestAccountsConfig, RpcLeaderScheduleConfig,
        RpcProgramAccountsConfig, RpcSendTransactionConfig, RpcSignatureStatusConfig, RpcSignaturesForAddressConfig,
        RpcSimulateTransactionConfig, RpcSupplyConfig, RpcTokenAccountsFilter, RpcTransactionConfig, Slot, Transaction,
        TransactionStatus, UiTokenAmount,
    },
};
use ic_solana_rpc::{
    auth::{do_authorize, do_deauthorize, require_manage_or_controller, require_register_provider, Auth},
    constants::NODES_IN_SUBNET,
    http::{get_http_request_cost, rpc_client, serve_logs, serve_metrics},
    providers::{do_register_provider, do_unregister_provider, do_update_provider},
    state::{read_state, replace_state, InitArgs},
    types::{RegisterProviderArgs, UpdateProviderArgs},
    utils::{parse_pubkey, parse_pubkeys, parse_signature, parse_signatures},
};

/// Returns all information associated with the account of the provided Pubkey.
#[update(name = "sol_getAccountInfo")]
#[candid_method(rename = "sol_getAccountInfo")]
pub async fn sol_get_account_info(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: Option<RpcAccountInfoConfig>,
) -> RpcResult<Option<UiAccount>> {
    let client = rpc_client(source, config);
    let pubkey = parse_pubkey(&pubkey)?;
    client
        .get_account_info(&pubkey, params)
        .await
        .map(|res| res.value.map(Into::into))
}

/// Returns the lamport balance of the account of provided Pubkey.
#[update(name = "sol_getBalance")]
#[candid_method(rename = "sol_getBalance")]
pub async fn sol_get_balance(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: Option<RpcContextConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    let pubkey = parse_pubkey(&pubkey)?;
    client.get_balance(&pubkey, params).await.map(|ctx| ctx.parse_value())
}

/// Returns identity and transaction information about a confirmed block in the ledger.
#[update(name = "sol_getBlock")]
#[candid_method(rename = "sol_getBlock")]
pub async fn sol_get_block(
    source: RpcServices,
    config: Option<RpcConfig>,
    slot: Slot,
    params: Option<RpcBlockConfig>,
) -> RpcResult<UiConfirmedBlock> {
    let client = rpc_client(source, config);
    client.get_block(slot, params).await.map(|ctx| ctx.into())
}

/// Returns commitment for a particular block.
#[update(name = "sol_getBlockCommitment")]
#[candid_method(rename = "sol_getBlockCommitment")]
pub async fn sol_get_block_commitment(
    source: RpcServices,
    config: Option<RpcConfig>,
    slot: Slot,
) -> RpcResult<RpcBlockCommitment> {
    let client = rpc_client(source, config);
    client.get_block_commitment(slot).await
}

/// Returns the current block height of the node.
#[update(name = "sol_getBlockHeight")]
#[candid_method(rename = "sol_getBlockHeight")]
pub async fn sol_get_block_height(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcContextConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client.get_block_height(params).await
}

/// Returns recent block production information from the current or previous epoch.
#[update(name = "sol_getBlockProduction")]
#[candid_method(rename = "sol_getBlockProduction")]
pub async fn sol_get_block_production(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcBlockProductionConfig>,
) -> RpcResult<RpcBlockProduction> {
    let client = rpc_client(source, config);
    client
        .get_block_production(params.map(Into::into))
        .await
        .map(|ctx| ctx.parse_value())
}

/// Returns the estimated production time of a block.
#[update(name = "sol_getBlockTime")]
#[candid_method(rename = "sol_getBlockTime")]
pub async fn sol_get_block_time(source: RpcServices, config: Option<RpcConfig>, slot: Slot) -> RpcResult<i64> {
    let client = rpc_client(source, config);
    client.get_block_time(slot).await
}

/// Returns a list of confirmed blocks between two slots.
#[update(name = "sol_getBlocks")]
#[candid_method(rename = "sol_getBlocks")]
pub async fn sol_get_blocks(
    source: RpcServices,
    config: Option<RpcConfig>,
    start_slot: Slot,
    last_slot: Option<Slot>,
    params: Option<CommitmentConfig>,
) -> RpcResult<Vec<u64>> {
    let client = rpc_client(source, config);
    client.get_blocks(start_slot, last_slot, params).await
}

/// Returns a list of confirmed blocks starting at the given slot.
#[update(name = "sol_getBlocksWithLimit")]
#[candid_method(rename = "sol_getBlocksWithLimit")]
pub async fn sol_get_blocks_with_limit(
    source: RpcServices,
    config: Option<RpcConfig>,
    start_slot: Slot,
    limit: u64,
    params: Option<CommitmentConfig>,
) -> RpcResult<Vec<u64>> {
    let client = rpc_client(source, config);
    client.get_blocks_with_limit(start_slot, limit, params).await
}

/// Returns information about all the nodes participating in the cluster.
#[update(name = "sol_getClusterNodes")]
#[candid_method(rename = "sol_getClusterNodes")]
pub async fn sol_get_cluster_nodes(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<Vec<RpcContactInfo>> {
    let client = rpc_client(source, config);
    client.get_cluster_nodes().await
}

/// Returns information about the current epoch.
#[update(name = "sol_getEpochInfo")]
#[candid_method(rename = "sol_getEpochInfo")]
pub async fn sol_get_epoch_info(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcContextConfig>,
) -> RpcResult<EpochInfo> {
    let client = rpc_client(source, config);
    client.get_epoch_info(params).await
}

/// Returns the epoch schedule information from this cluster's genesis config.
#[update(name = "sol_getEpochSchedule")]
#[candid_method(rename = "sol_getEpochSchedule")]
pub async fn sol_get_epoch_schedule(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<EpochSchedule> {
    let client = rpc_client(source, config);
    client.get_epoch_schedule().await
}

/// Get the fee the network will charge for a particular Message.
#[update(name = "sol_getFeeForMessage")]
#[candid_method(rename = "sol_getFeeForMessage")]
pub async fn sol_get_fee_for_message(
    source: RpcServices,
    config: Option<RpcConfig>,
    message: String,
    params: Option<RpcContextConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client
        .get_fee_for_message(message, params)
        .await
        .map(|ctx| ctx.parse_value())
}

/// Returns the slot of the lowest confirmed block that has not been purged from the ledger.
#[update(name = "sol_getFirstAvailableBlock")]
#[candid_method(rename = "sol_getFirstAvailableBlock")]
pub async fn sol_get_first_available_block(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<Slot> {
    let client = rpc_client(source, config);
    client.get_first_available_block().await
}

/// Returns the genesis hash.
#[update(name = "sol_getGenesisHash")]
#[candid_method(rename = "sol_getGenesisHash")]
pub async fn sol_get_genesis_hash(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<String> {
    let client = rpc_client(source, config);
    client.get_genesis_hash().await
}

/// Returns the current health of the node.
/// A healthy node is one that is within HEALTH_CHECK_SLOT_DISTANCE slots of
/// the latest cluster-confirmed slot.
#[update(name = "sol_getHealth")]
#[candid_method(rename = "sol_getHealth")]
pub async fn sol_get_health(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<String> {
    let client = rpc_client(source, config);
    client.get_health().await
}

/// Returns the highest slot information that the node has snapshots for.
/// This will find the highest full snapshot slot and the highest incremental
/// snapshot slot based on the full snapshot slot, if there is one.
#[update(name = "sol_getHighestSnapshotSlot")]
#[candid_method(rename = "sol_getHighestSnapshotSlot")]
pub async fn sol_get_highest_snapshot_slot(
    source: RpcServices,
    config: Option<RpcConfig>,
) -> RpcResult<RpcSnapshotSlotInfo> {
    let client = rpc_client(source, config);
    client.get_highest_snapshot_slot().await
}

/// Returns the identity pubkey for the current node.
#[update(name = "sol_getIdentity")]
#[candid_method(rename = "sol_getIdentity")]
pub async fn sol_get_identity(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<RpcIdentity> {
    let client = rpc_client(source, config);
    client.get_identity().await
}

/// Returns the current inflation governor.
#[update(name = "sol_getInflationGovernor")]
#[candid_method(rename = "sol_getInflationGovernor")]
pub async fn sol_get_inflation_governor(
    source: RpcServices,
    config: Option<RpcConfig>,
) -> RpcResult<RpcInflationGovernor> {
    let client = rpc_client(source, config);
    client.get_inflation_governor().await
}

/// Returns the specific inflation values for the current epoch.
#[update(name = "sol_getInflationRate")]
#[candid_method(rename = "sol_getInflationRate")]
pub async fn sol_get_inflation_rate(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<RpcInflationRate> {
    let client = rpc_client(source, config);
    client.get_inflation_rate().await
}

/// Returns the inflation / staking reward for a list of addresses for an epoch.
#[update(name = "sol_getInflationReward")]
#[candid_method(rename = "sol_getInflationReward")]
pub async fn sol_get_inflation_reward(
    source: RpcServices,
    config: Option<RpcConfig>,
    addresses: Vec<String>,
    params: Option<RpcEpochConfig>,
) -> RpcResult<Vec<Option<RpcInflationReward>>> {
    let client = rpc_client(source, config);
    let pubkeys = parse_pubkeys(addresses)?;
    client.get_inflation_reward(&pubkeys, params).await
}

/// Returns signatures for confirmed transactions that
/// include the given address in their accountKeys list.
#[update(name = "sol_getSignaturesForAddress")]
#[candid_method(rename = "sol_getSignaturesForAddress")]
pub async fn sol_get_signatures_for_address(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: Option<RpcSignaturesForAddressConfig>,
) -> RpcResult<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    let client = rpc_client(source, config);
    let pubkey = parse_pubkey(&pubkey)?;
    client.get_signatures_for_address(&pubkey, params).await
}

/// Returns the slot that has reached the given or default commitment level.
#[update(name = "sol_getSlot")]
#[candid_method(rename = "sol_getSlot")]
pub async fn sol_get_slot(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcContextConfig>,
) -> RpcResult<Slot> {
    let client = rpc_client(source, config);
    client.get_slot(params).await
}

/// Returns the current slot leader.
#[update(name = "sol_getSlotLeader")]
#[candid_method(rename = "sol_getSlotLeader")]
pub async fn sol_get_slot_leader(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcContextConfig>,
) -> RpcResult<String> {
    let client = rpc_client(source, config);
    client.get_slot_leader(params).await
}

/// Returns the slot leaders for a given slot range.
#[update(name = "sol_getSlotLeaders")]
#[candid_method(rename = "sol_getSlotLeaders")]
pub async fn sol_get_slot_leaders(
    source: RpcServices,
    config: Option<RpcConfig>,
    start_slot: u64,
    limit: Option<u64>,
) -> RpcResult<Vec<String>> {
    let client = rpc_client(source, config);
    client.get_slot_leaders(start_slot, limit).await
}

/// Returns the stake minimum delegation, in lamports.
#[update(name = "sol_getStakeMinimumDelegation")]
#[candid_method(rename = "sol_getStakeMinimumDelegation")]
pub async fn sol_get_stake_minimum_delegation(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<CommitmentConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    Ok(client.get_stake_minimum_delegation(params).await?.value)
}

/// Returns information about the current supply.
#[update(name = "sol_getSupply")]
#[candid_method(rename = "sol_getSupply")]
pub async fn sol_get_supply(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcSupplyConfig>,
) -> RpcResult<RpcSupply> {
    let client = rpc_client(source, config);
    Ok(client.get_supply(params).await?.value)
}

/// Returns the token balance of an SPL Token account.
#[update(name = "sol_getTokenAccountBalance")]
#[candid_method(rename = "sol_getTokenAccountBalance")]
pub async fn sol_get_token_account_balance(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    commitment: Option<CommitmentLevel>,
) -> RpcResult<UiTokenAmount> {
    let client = rpc_client(source, config);
    let pubkey = parse_pubkey(&pubkey)?;
    Ok(client
        .get_token_account_balance(&pubkey, commitment.map(Into::into))
        .await?
        .parse_value())
}

/// Returns all SPL Token accounts by approved Delegate.
#[update(name = "sol_getTokenAccountsByDelegate")]
#[candid_method(rename = "sol_getTokenAccountsByDelegate")]
pub async fn sol_get_token_accounts_by_delegate(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    filter: RpcTokenAccountsFilter,
    params: Option<RpcAccountInfoConfig>,
) -> RpcResult<Vec<RpcKeyedAccount>> {
    let client = rpc_client(source, config);
    let pubkey = parse_pubkey(&pubkey)?;
    let accounts = client
        .get_token_accounts_by_delegate(&pubkey, filter, params)
        .await?
        .parse_value();
    Ok(accounts.into_iter().map(Into::into).collect())
}

/// Returns all SPL Token accounts by token owner.
#[update(name = "sol_getTokenAccountsByOwner")]
#[candid_method(rename = "sol_getTokenAccountsByOwner")]
pub async fn sol_get_token_accounts_by_owner(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    filter: RpcTokenAccountsFilter,
    params: Option<RpcAccountInfoConfig>,
) -> RpcResult<Vec<RpcKeyedAccount>> {
    let client = rpc_client(source, config);
    let pubkey = parse_pubkey(&pubkey)?;
    let accounts = client
        .get_token_accounts_by_owner(&pubkey, filter, params)
        .await?
        .parse_value();
    Ok(accounts.into_iter().map(Into::into).collect())
}

/// Returns the 20 largest accounts of a particular SPL Token type.
#[update(name = "sol_getTokenLargestAccounts")]
#[candid_method(rename = "sol_getTokenLargestAccounts")]
pub async fn sol_get_token_largest_accounts(
    source: RpcServices,
    config: Option<RpcConfig>,
    mint: String,
    params: Option<CommitmentConfig>,
) -> RpcResult<Vec<RpcTokenAccountBalance>> {
    let client = rpc_client(source, config);
    let mint = parse_pubkey(&mint)?;
    let accounts = client.get_token_largest_accounts(&mint, params).await?.parse_value();
    Ok(accounts.into_iter().map(Into::into).collect())
}

/// Returns the total supply of an SPL Token type.
#[update(name = "sol_getTokenSupply")]
#[candid_method(rename = "sol_getTokenSupply")]
pub async fn sol_get_token_supply(
    source: RpcServices,
    config: Option<RpcConfig>,
    mint: String,
    params: Option<CommitmentConfig>,
) -> RpcResult<UiTokenAmount> {
    let client = rpc_client(source, config);
    let mint = parse_pubkey(&mint)?;
    Ok(client.get_token_supply(&mint, params).await?.parse_value())
}

/// Returns the 20 largest accounts, by lamport balance (results may be cached up to two hours).
#[update(name = "sol_getLargestAccounts")]
#[candid_method(rename = "sol_getLargestAccounts")]
pub async fn sol_get_largest_accounts(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcLargestAccountsConfig>,
) -> RpcResult<Vec<RpcAccountBalance>> {
    let client = rpc_client(source, config);
    Ok(client.get_largest_accounts(params).await?.parse_value())
}

/// Returns the latest blockhash.
#[update(name = "sol_getLatestBlockhash")]
#[candid_method(rename = "sol_getLatestBlockhash")]
pub async fn sol_get_latest_blockhash(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcContextConfig>,
) -> RpcResult<RpcBlockhash> {
    let client = rpc_client(source, config);
    Ok(client.get_latest_blockhash(params).await?.parse_value())
}

/// Returns the leader schedule for an epoch.
#[update(name = "sol_getLeaderSchedule")]
#[candid_method(rename = "sol_getLeaderSchedule")]
pub async fn sol_get_leader_schedule(
    source: RpcServices,
    config: Option<RpcConfig>,
    epoch: u64,
    params: Option<RpcLeaderScheduleConfig>,
) -> RpcResult<RpcLeaderSchedule> {
    let client = rpc_client(source, config);
    client.get_leader_schedule(epoch, params).await
}

/// Get the max slot seen from the retransmit stage.
#[update(name = "sol_getMaxRetransmitSlot")]
#[candid_method(rename = "sol_getMaxRetransmitSlot")]
pub async fn sol_get_max_retransmit_slot(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client.get_max_retransmit_slot().await
}

/// Get the max slot seen from after shred insert.
#[update(name = "sol_getMaxShredInsertSlot")]
#[candid_method(rename = "sol_getMaxShredInsertSlot")]
pub async fn sol_get_max_shred_insert_slot(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client.get_max_shred_insert_slot().await
}

/// Returns the minimum balance required to make account rent exempt.
#[update(name = "sol_getMinimumBalanceForRentExemption")]
#[candid_method(rename = "sol_getMinimumBalanceForRentExemption")]
pub async fn sol_get_minimum_balance_for_rent_exemption(
    source: RpcServices,
    config: Option<RpcConfig>,
    size: usize,
    params: Option<CommitmentConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client.get_minimum_balance_for_rent_exemption(size, params).await
}

/// Returns the account information for a list of Pubkeys.
#[update(name = "sol_getMultipleAccounts")]
#[candid_method(rename = "sol_getMultipleAccounts")]
pub async fn sol_get_multiple_accounts(
    source: RpcServices,
    config: Option<RpcConfig>,
    addresses: Vec<String>,
    params: Option<RpcAccountInfoConfig>,
) -> RpcResult<Vec<UiAccount>> {
    let client = rpc_client(source, config);
    let pubkeys = parse_pubkeys(addresses)?;
    let res = client.get_multiple_accounts(pubkeys, params).await?.parse_value();
    Ok(res.into_iter().map(Into::into).collect())
}

/// Returns all accounts owned by the provided program Pubkey.
#[update(name = "sol_getProgramAccounts")]
#[candid_method(rename = "sol_getProgramAccounts")]
pub async fn sol_get_program_accounts(
    source: RpcServices,
    config: Option<RpcConfig>,
    program: String,
    params: Option<RpcProgramAccountsConfig>,
) -> RpcResult<Vec<RpcKeyedAccount>> {
    let pubkey = parse_pubkey(&program)?;
    let client = rpc_client(source, config);
    let res = client.get_program_accounts(&pubkey, params).await?;
    Ok(res.into_iter().map(Into::into).collect())
}

/// Returns a list of recent performance samples, in reverse slot order.
/// Performance samples are taken every 60 seconds and include the number
/// of transactions and slots that occur in a given time window.
#[update(name = "sol_getRecentPerformanceSamples")]
#[candid_method(rename = "sol_getRecentPerformanceSamples")]
pub async fn sol_get_recent_performance_samples(
    source: RpcServices,
    config: Option<RpcConfig>,
    limit: u64,
) -> RpcResult<Vec<RpcPerfSample>> {
    let client = rpc_client(source, config);
    client.get_recent_performance_samples(limit).await
}

/// Returns a list of prioritization fees from recent blocks.
#[update(name = "sol_getRecentPrioritizationFees")]
#[candid_method(rename = "sol_getRecentPrioritizationFees")]
pub async fn sol_get_recent_prioritization_fees(
    source: RpcServices,
    config: Option<RpcConfig>,
    addresses: Vec<String>,
) -> RpcResult<Vec<RpcPrioritizationFee>> {
    let client = rpc_client(source, config);
    let pubkeys = parse_pubkeys(addresses)?;
    client.get_recent_prioritization_fees(&pubkeys).await
}

/// Returns the statuses of a list of signatures.
/// Each signature must be a txid, the first signature of a transaction.
#[update(name = "sol_getSignatureStatuses")]
#[candid_method(rename = "sol_getSignatureStatuses")]
pub async fn sol_get_signature_statuses(
    source: RpcServices,
    config: Option<RpcConfig>,
    signatures: Vec<String>,
    params: Option<RpcSignatureStatusConfig>,
) -> RpcResult<Vec<Option<TransactionStatus>>> {
    let client = rpc_client(source, config);
    let signatures = parse_signatures(signatures)?;
    Ok(client.get_signature_statuses(&signatures, params).await?.parse_value())
}

/// Returns transaction details for a confirmed transaction.
#[update(name = "sol_getTransaction")]
#[candid_method(rename = "sol_getTransaction")]
pub async fn sol_get_transaction(
    source: RpcServices,
    config: Option<RpcConfig>,
    signature: String,
    params: Option<RpcTransactionConfig>,
) -> RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>> {
    let client = rpc_client(source, config);
    let signature = parse_signature(&signature)?;
    let response = client.get_transaction(&signature, params).await?;
    Ok(response.map(|tx| tx.into()))
}

/// Returns the current number of transactions from the ledger.
#[update(name = "sol_getTransactionCount")]
#[candid_method(rename = "sol_getTransactionCount")]
pub async fn sol_get_transaction_count(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcContextConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client.get_transaction_count(params).await
}

/// Returns the current Solana version running on the node.
#[update(name = "sol_getVersion")]
#[candid_method(rename = "sol_getVersion")]
pub async fn sol_get_version(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<RpcVersionInfo> {
    let client = rpc_client(source, config);
    client.get_version().await
}

/// Returns the account info and associated stake for all the voting accounts in the current bank.
#[update(name = "sol_getVoteAccounts")]
#[candid_method(rename = "sol_getVoteAccounts")]
pub async fn sol_get_vote_accounts(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcGetVoteAccountsConfig>,
) -> RpcResult<RpcVoteAccountStatus> {
    let client = rpc_client(source, config);
    client.get_vote_accounts(params).await
}

/// Returns whether a blockhash is still valid or not.
#[update(name = "sol_isBlockhashValid")]
#[candid_method(rename = "sol_isBlockhashValid")]
pub async fn sol_is_blockhash_valid(
    source: RpcServices,
    config: Option<RpcConfig>,
    blockhash: String,
    params: Option<RpcContextConfig>,
) -> RpcResult<bool> {
    let client = rpc_client(source, config);
    Ok(client.is_blockhash_valid(blockhash, params).await?.parse_value())
}

/// Returns the lowest slot that the node has information about in its ledger.
#[update(name = "sol_minimumLedgerSlot")]
#[candid_method(rename = "sol_minimumLedgerSlot")]
pub async fn sol_minimum_ledger_slot(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client.minimum_ledger_slot().await
}

/// Requests an airdrop of lamports to a Pubkey.
#[update(name = "sol_requestAirdrop")]
#[candid_method(rename = "sol_requestAirdrop")]
pub async fn sol_request_airdrop(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    lamports: u64,
) -> RpcResult<String> {
    let client = rpc_client(source, config);
    let pubkey = parse_pubkey(&pubkey)?;
    client.request_airdrop(&pubkey, lamports).await
}

/// Submits a signed transaction to the cluster for processing.
/// Use `sol_getSignatureStatuses` to ensure a transaction is processed and confirmed.
#[update(name = "sol_sendTransaction")]
#[candid_method(rename = "sol_sendTransaction")]
pub async fn sol_send_transaction(
    source: RpcServices,
    config: Option<RpcConfig>,
    raw_signed_transaction: String,
    params: Option<RpcSendTransactionConfig>,
) -> RpcResult<String> {
    let client = rpc_client(source, config);
    let tx = Transaction::from_str(&raw_signed_transaction).expect("Invalid transaction");
    let signature = client.send_transaction(tx, params.unwrap_or_default()).await?;
    Ok(signature.to_string())
}

/// Simulate sending a transaction.
#[update(name = "sol_simulateTransaction")]
#[candid_method(rename = "sol_simulateTransaction")]
pub async fn sol_simulate_transaction(
    source: RpcServices,
    config: Option<RpcConfig>,
    raw_transaction: String,
    params: Option<RpcSimulateTransactionConfig>,
) -> RpcResult<RpcSimulateTransactionResult> {
    let client = rpc_client(source, config);
    let tx = Transaction::from_str(&raw_transaction).expect("Invalid transaction");
    let res = client.simulate_transaction(tx, params.unwrap_or_default()).await?;
    Ok(res.parse_value().into())
}

/// Retrieves transaction logs for a given public key.
///
/// This function fetches transaction signatures associated with the provided `pubkey` and then
/// retrieves detailed transaction data based on those signatures.
#[update(name = "sol_getLogs")]
#[candid_method(rename = "sol_getLogs")]
pub async fn sol_get_logs(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: Option<RpcSignaturesForAddressConfig>,
) -> RpcResult<HashMap<String, RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>>> {
    let client = rpc_client(source, config);
    let pubkey = parse_pubkey(&pubkey)?;
    let commitment = params.as_ref().and_then(|p| p.commitment);
    let signatures = client.get_signatures_for_address(&pubkey, params).await?;

    let transactions = client
        .get_transactions(
            signatures.iter().map(|s| s.signature.as_str()).collect(),
            Some(RpcTransactionConfig {
                commitment,
                ..Default::default()
            }),
        )
        .await?;

    Ok(transactions
        .into_iter()
        .map(|(k, v)| (k, v.map(|opt| opt.map(Into::into))))
        .collect())
}

/// Sends a JSON-RPC request to a specified Solana node provider,
/// supporting custom RPC methods.
#[update]
#[candid_method]
pub async fn request(
    source: RpcServices,
    method: String,
    params: CandidValue,
    max_response_bytes: Option<u64>,
) -> RpcResult<String> {
    let client = rpc_client(source, None);
    let res = client
        .call::<_, serde_json::Value>(RpcRequest::Custom { method }, params, max_response_bytes)
        .await?;
    Ok(serde_json::to_string(&res)?)
}

/// Calculates the cost of an RPC request.
#[query(name = "requestCost")]
#[candid_method(query, rename = "requestCost")]
pub fn request_cost(json_rpc_payload: String, max_response_bytes: u64) -> u128 {
    if read_state(|s| s.is_demo_active) {
        0
    } else {
        get_http_request_cost(json_rpc_payload.len() as u64, max_response_bytes)
    }
}

#[query(name = "getNodesInSubnet")]
#[candid_method(query, rename = "getNodesInSubnet")]
fn get_nodes_in_subnet() -> u32 {
    NODES_IN_SUBNET
}

#[query(name = "getProviders")]
#[candid_method(query, rename = "getProviders")]
fn get_providers() -> Vec<String> {
    read_state(|s| s.rpc_providers.iter().map(|(k, _)| k.0).collect())
}

#[update(name = "registerProvider", guard = "require_register_provider")]
#[candid_method(rename = "registerProvider")]
fn register_provider(args: RegisterProviderArgs) {
    do_register_provider(ic_cdk::caller(), args)
}

#[update(name = "unregisterProvider")]
#[candid_method(rename = "unregisterProvider")]
fn unregister_provider(provider_id: String) -> bool {
    do_unregister_provider(ic_cdk::caller(), &provider_id)
}

#[update(name = "updateProvider")]
#[candid_method(rename = "updateProvider")]
fn update_provider(args: UpdateProviderArgs) {
    do_update_provider(ic_cdk::caller(), args)
}

#[query(name = "getAuthorized")]
#[candid_method(query, rename = "getAuthorized")]
fn get_authorized(auth: Auth) -> Vec<Principal> {
    read_state(|s| {
        let mut result = Vec::new();
        for (k, v) in s.auth.iter() {
            if v.is_authorized(auth) {
                result.push(k.0);
            }
        }
        result
    })
}

#[update(guard = "require_manage_or_controller")]
#[candid_method]
fn authorize(principal: Principal, auth: Auth) -> bool {
    do_authorize(principal, auth)
}

#[update(guard = "require_manage_or_controller")]
#[candid_method]
fn deauthorize(principal: Principal, auth: Auth) -> bool {
    do_deauthorize(principal, auth)
}

#[query]
fn http_request(request: AssetHttpRequest) -> AssetHttpResponse {
    match request.path() {
        "/metrics" => serve_metrics(encode_metrics),
        "/logs" => serve_logs(request),
        _ => HttpResponseBuilder::not_found().build(),
    }
}

#[query(name = "getMetrics")]
#[candid_method(query, rename = "getMetrics")]
fn get_metrics() -> Metrics {
    read_metrics(|m| m.to_owned())
}

/// Cleans up the HTTP response headers to make them deterministic.
///
/// # Arguments
///
/// * `args` - Transformation arguments containing the HTTP response.
#[query(hidden = true)]
fn __transform_json_rpc(mut args: TransformArgs) -> HttpResponse {
    // The response header contains non-deterministic fields that make it impossible to reach
    // consensus! Errors seem deterministic and do not contain data that can break consensus.
    // Clear non-deterministic fields from the response headers.
    args.response.headers.clear();
    args.response
}

#[ic_cdk::init]
fn init(args: InitArgs) {
    post_upgrade(args)
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: InitArgs) {
    replace_state(args.into());
}

fn main() {}

// Order dependent: do not move above any exposed canister method!
ic_cdk::export_candid!();

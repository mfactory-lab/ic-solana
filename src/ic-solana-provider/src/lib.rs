use {
    crate::{
        auth::{do_authorize, do_deauthorize, require_manage_or_controller, require_register_provider, Auth},
        constants::NODES_IN_SUBNET,
        http::{get_http_request_cost, rpc_client, serve_logs, serve_metrics},
        providers::{do_register_provider, do_unregister_provider, do_update_provider},
        state::STATE,
        types::{RegisterProviderArgs, RpcConfig, RpcServices, UpdateProviderArgs},
    },
    candid::{candid_method, Principal},
    ic_canisters_http_types::{
        HttpRequest as AssetHttpRequest, HttpResponse as AssetHttpResponse, HttpResponseBuilder,
    },
    ic_cdk::{
        api::management_canister::http_request::{HttpResponse, TransformArgs},
        query, update,
    },
    ic_solana::{
        request::RpcRequest,
        response::{
            RpcBlockCommitment, RpcBlockProduction, RpcConfirmedTransactionStatusWithSignature, RpcContactInfo,
            RpcIdentity, RpcInflationGovernor, RpcInflationRate, RpcInflationReward, RpcSnapshotSlotInfo, RpcSupply,
            RpcVersionInfo, RpcVoteAccountStatus,
        },
        rpc_client::{RpcError, RpcResult},
        types::{
            Account, CandidValue, CommitmentConfig, EncodedConfirmedTransactionWithStatusMeta, EpochInfo,
            EpochSchedule, Pubkey, RpcAccountInfoConfig, RpcBlockConfig, RpcContextConfig, RpcEpochConfig,
            RpcGetVoteAccountsConfig, RpcSendTransactionConfig, RpcSignatureStatusConfig,
            RpcSignaturesForAddressConfig, RpcSupplyConfig, RpcTokenAccountsFilter, RpcTransactionConfig, Signature,
            Slot, TaggedEncodedConfirmedTransactionWithStatusMeta, TaggedRpcBlockProductionConfig,
            TaggedRpcKeyedAccount, TaggedRpcTokenAccountBalance, TaggedUiConfirmedBlock, Transaction,
            TransactionStatus, UiTokenAmount,
        },
    },
    ic_solana_common::metrics::{encode_metrics, read_metrics, Metrics},
    state::{read_state, InitArgs},
    std::{collections::HashMap, str::FromStr},
};

pub mod auth;
mod constants;
mod http;
mod memory;
mod providers;
pub mod state;
pub mod types;
mod utils;

///
/// Returns all information associated with the account of the provided Pubkey.
///
#[update(name = "sol_getAccountInfo")]
#[candid_method(rename = "sol_getAccountInfo")]
pub async fn sol_get_account_info(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: Option<RpcAccountInfoConfig>,
) -> RpcResult<Option<Account>> {
    let client = rpc_client(source, config);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let account_info = client.get_account_info(&pubkey, params).await?;
    Ok(account_info)
}

///
/// Returns the lamport balance of the account of provided Pubkey.
///
#[update(name = "sol_getBalance")]
#[candid_method(rename = "sol_getBalance")]
pub async fn sol_get_balance(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: Option<RpcContextConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let balance = client.get_balance(&pubkey, params).await?;
    Ok(balance)
}

///
/// Returns identity and transaction information about a confirmed block in the ledger.
///
#[update(name = "sol_getBlock")]
#[candid_method(rename = "sol_getBlock")]
pub async fn sol_get_block(
    source: RpcServices,
    config: Option<RpcConfig>,
    slot: Slot,
    params: Option<RpcBlockConfig>,
) -> RpcResult<TaggedUiConfirmedBlock> {
    let client = rpc_client(source, config);
    let block = client.get_block(slot, params).await?;
    Ok(block.into())
}

///
/// Returns commitment for a particular block.
///
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

///
/// Returns the current block height of the node.
///
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

///
/// Returns recent block production information from the current or previous epoch.
///
#[update(name = "sol_getBlockProduction")]
#[candid_method(rename = "sol_getBlockProduction")]
pub async fn sol_get_block_production(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: TaggedRpcBlockProductionConfig,
) -> RpcResult<RpcBlockProduction> {
    let client = rpc_client(source, config);
    client.get_block_production(params.into()).await
}

///
/// Returns the estimated production time of a block.
///
#[update(name = "sol_getBlockTime")]
#[candid_method(rename = "sol_getBlockTime")]
pub async fn sol_get_block_time(source: RpcServices, config: Option<RpcConfig>, slot: Slot) -> RpcResult<i64> {
    let client = rpc_client(source, config);
    client.get_block_time(slot).await
}

///
/// Returns a list of confirmed blocks between two slots.
///
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

///
/// Returns a list of confirmed blocks starting at the given slot.
///
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

///
/// Returns information about all the nodes participating in the cluster.
///
#[update(name = "sol_getClusterNodes")]
#[candid_method(rename = "sol_getClusterNodes")]
pub async fn sol_get_cluster_nodes(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<Vec<RpcContactInfo>> {
    let client = rpc_client(source, config);
    client.get_cluster_nodes().await
}

///
/// Returns information about the current epoch.
///
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

///
/// Returns the epoch schedule information from this cluster's genesis config.
///
#[update(name = "sol_getEpochSchedule")]
#[candid_method(rename = "sol_getEpochSchedule")]
pub async fn sol_get_epoch_schedule(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<EpochSchedule> {
    let client = rpc_client(source, config);
    client.get_epoch_schedule().await
}

///
/// Get the fee the network will charge for a particular Message.
///
#[update(name = "sol_getFeeForMessage")]
#[candid_method(rename = "sol_getFeeForMessage")]
pub async fn sol_get_fee_for_message(
    source: RpcServices,
    config: Option<RpcConfig>,
    message: String,
    params: Option<RpcContextConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client.get_fee_for_message(message, params).await
}

///
/// Returns the slot of the lowest confirmed block that has not been purged from the ledger.
///
#[update(name = "sol_getFirstAvailableBlock")]
#[candid_method(rename = "sol_getFirstAvailableBlock")]
pub async fn sol_get_first_available_block(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<Slot> {
    let client = rpc_client(source, config);
    client.get_first_available_block().await
}

///
/// Returns the genesis hash.
///
#[update(name = "sol_getGenesisHash")]
#[candid_method(rename = "sol_getGenesisHash")]
pub async fn sol_get_genesis_hash(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<String> {
    let client = rpc_client(source, config);
    client.get_genesis_hash().await
}

///
/// Returns the current health of the node.
/// A healthy node is one that is within HEALTH_CHECK_SLOT_DISTANCE slots of
/// the latest cluster-confirmed slot.
///
#[update(name = "sol_getHealth")]
#[candid_method(rename = "sol_getHealth")]
pub async fn sol_get_health(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<String> {
    let client = rpc_client(source, config);
    client.get_health().await
}

///
/// Returns the highest slot information that the node has snapshots for.
/// This will find the highest full snapshot slot and the highest incremental
/// snapshot slot based on the full snapshot slot, if there is one.
///
#[update(name = "sol_getHighestSnapshotSlot")]
#[candid_method(rename = "sol_getHighestSnapshotSlot")]
pub async fn sol_get_highest_snapshot_slot(
    source: RpcServices,
    config: Option<RpcConfig>,
) -> RpcResult<RpcSnapshotSlotInfo> {
    let client = rpc_client(source, config);
    client.get_highest_snapshot_slot().await
}

///
/// Returns the identity pubkey for the current node.
///
#[update(name = "sol_getIdentity")]
#[candid_method(rename = "sol_getIdentity")]
pub async fn sol_get_identity(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<RpcIdentity> {
    let client = rpc_client(source, config);
    client.get_identity().await
}

///
/// Returns the current inflation governor.
///
#[update(name = "sol_getInflationGovernor")]
#[candid_method(rename = "sol_getInflationGovernor")]
pub async fn sol_get_inflation_governor(
    source: RpcServices,
    config: Option<RpcConfig>,
) -> RpcResult<RpcInflationGovernor> {
    let client = rpc_client(source, config);
    client.get_inflation_governor().await
}

///
/// Returns the specific inflation values for the current epoch.
///
#[update(name = "sol_getInflationRate")]
#[candid_method(rename = "sol_getInflationRate")]
pub async fn sol_get_inflation_rate(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<RpcInflationRate> {
    let client = rpc_client(source, config);
    client.get_inflation_rate().await
}

///
/// Returns the inflation / staking reward for a list of addresses for an epoch.
///
#[update(name = "sol_getInflationReward")]
#[candid_method(rename = "sol_getInflationReward")]
pub async fn sol_get_inflation_reward(
    source: RpcServices,
    config: Option<RpcConfig>,
    addresses: Vec<String>,
    params: RpcEpochConfig,
) -> RpcResult<Vec<Option<RpcInflationReward>>> {
    let client = rpc_client(source, config);
    let pubkeys = addresses
        .iter()
        .map(|x| Pubkey::from_str(x).map_err(|e| RpcError::ParseError(e.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    client.get_inflation_reward(&pubkeys, params).await
}

///
/// Returns signatures for confirmed transactions that
/// include the given address in their accountKeys list.
///
#[update(name = "sol_getSignaturesForAddress")]
#[candid_method(rename = "sol_getSignaturesForAddress")]
pub async fn sol_get_signatures_for_address(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: RpcSignaturesForAddressConfig,
) -> RpcResult<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    let client = rpc_client(source, config);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let result = client.get_signatures_for_address(&pubkey, params).await?;
    Ok(result)
}

///
/// Returns the slot that has reached the given or default commitment level.
///
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

///
/// Returns the current slot leader.
///
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

///
/// Returns the slot leaders for a given slot range.
///
#[update(name = "sol_getSlotLeaders")]
#[candid_method(rename = "sol_getSlotLeaders")]
pub async fn sol_get_slot_leaders(
    source: RpcServices,
    config: Option<RpcConfig>,
    start_slot: u64,
    limit: Option<u64>,
) -> RpcResult<String> {
    let client = rpc_client(source, config);
    client.get_slot_leaders(start_slot, limit).await
}

///
/// Returns the stake minimum delegation, in lamports.
///
#[update(name = "sol_getStakeMinimumDelegation")]
#[candid_method(rename = "sol_getStakeMinimumDelegation")]
pub async fn sol_get_stake_minimum_delegation(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<CommitmentConfig>,
) -> RpcResult<u64> {
    let client = rpc_client(source, config);
    client.get_stake_minimum_delegation(params).await.map(|c| c.value)
}

///
/// Returns information about the current supply.
///
#[update(name = "sol_getSupply")]
#[candid_method(rename = "sol_getSupply")]
pub async fn sol_get_supply(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: RpcSupplyConfig,
) -> RpcResult<RpcSupply> {
    let client = rpc_client(source, config);
    client.get_supply(params).await
}

///
/// Returns the token balance of an SPL Token account.
///
#[update(name = "sol_getTokenAccountBalance")]
#[candid_method(rename = "sol_getTokenAccountBalance")]
pub async fn sol_get_token_account_balance(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: Option<CommitmentConfig>,
) -> RpcResult<UiTokenAmount> {
    let client = rpc_client(source, config);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    client.get_token_account_balance(&pubkey, params).await
}

///
/// Returns all SPL Token accounts by approved Delegate.
///
#[update(name = "sol_getTokenAccountsByDelegate")]
#[candid_method(rename = "sol_getTokenAccountsByDelegate")]
pub async fn sol_get_token_accounts_by_delegate(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    filter: RpcTokenAccountsFilter,
    params: Option<RpcAccountInfoConfig>,
) -> RpcResult<Vec<TaggedRpcKeyedAccount>> {
    let client = rpc_client(source, config);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let accounts = client.get_token_accounts_by_delegate(&pubkey, filter, params).await?;
    Ok(accounts.into_iter().map(Into::into).collect())
}

///
/// Returns all SPL Token accounts by token owner.
///
#[update(name = "sol_getTokenAccountsByOwner")]
#[candid_method(rename = "sol_getTokenAccountsByOwner")]
pub async fn sol_get_token_accounts_by_owner(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    filter: RpcTokenAccountsFilter,
    params: Option<RpcAccountInfoConfig>,
) -> RpcResult<Vec<TaggedRpcKeyedAccount>> {
    let client = rpc_client(source, config);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let accounts = client.get_token_accounts_by_owner(&pubkey, filter, params).await?;

    Ok(accounts.into_iter().map(Into::into).collect())
}

///
/// Returns the 20 largest accounts of a particular SPL Token type.
///
#[update(name = "sol_getTokenLargestAccounts")]
#[candid_method(rename = "sol_getTokenLargestAccounts")]
pub async fn sol_get_token_largest_accounts(
    source: RpcServices,
    config: Option<RpcConfig>,
    mint: String,
    params: Option<CommitmentConfig>,
) -> RpcResult<Vec<TaggedRpcTokenAccountBalance>> {
    let client = rpc_client(source, config);
    let mint = Pubkey::from_str(&mint).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let accounts = client.get_token_largest_accounts(&mint, params).await?;

    Ok(accounts.into_iter().map(Into::into).collect())
}

///
/// Returns the total supply of an SPL Token type.
///
#[update(name = "sol_getTokenSupply")]
#[candid_method(rename = "sol_getTokenSupply")]
pub async fn sol_get_token_supply(
    source: RpcServices,
    config: Option<RpcConfig>,
    mint: String,
    params: Option<CommitmentConfig>,
) -> RpcResult<UiTokenAmount> {
    let client = rpc_client(source, config);
    let mint = Pubkey::from_str(&mint).map_err(|e| RpcError::ParseError(e.to_string()))?;
    client.get_token_supply(&mint, params).await
}

///
/// Returns the latest blockhash.
///
#[update(name = "sol_getLatestBlockhash")]
#[candid_method(rename = "sol_getLatestBlockhash")]
pub async fn sol_get_latest_blockhash(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: Option<RpcContextConfig>,
) -> RpcResult<String> {
    let client = rpc_client(source, config);
    let blockhash = client.get_latest_blockhash(params).await?;
    Ok(blockhash.to_string())
}

///
/// Returns the statuses of a list of signatures.
/// Each signature must be a txid, the first signature of a transaction.
///
#[update(name = "sol_getSignatureStatuses")]
#[candid_method(rename = "sol_getSignatureStatuses")]
pub async fn sol_get_signature_statuses(
    source: RpcServices,
    config: Option<RpcConfig>,
    signatures: Vec<String>,
    params: Option<RpcSignatureStatusConfig>,
) -> RpcResult<Vec<Option<TransactionStatus>>> {
    let client = rpc_client(source, config);

    let signatures: Vec<Signature> = signatures
        .into_iter()
        .map(|s| Signature::from_str(&s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| RpcError::ParseError(e.to_string()))?;

    client.get_signature_statuses(&signatures, params).await
}

///
/// Returns transaction details for a confirmed transaction.
///
#[update(name = "sol_getTransaction")]
#[candid_method(rename = "sol_getTransaction")]
pub async fn sol_get_transaction(
    source: RpcServices,
    config: Option<RpcConfig>,
    signature: String,
    params: Option<RpcTransactionConfig>,
) -> RpcResult<TaggedEncodedConfirmedTransactionWithStatusMeta> {
    let client = rpc_client(source, config);
    let signature = Signature::from_str(&signature).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let response = client.get_transaction(&signature, params).await?;
    Ok(response.into())
}

///
/// Returns the current Transaction count from the ledger.
///
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

///
/// Returns the current Solana version running on the node.
///
#[update(name = "sol_getVersion")]
#[candid_method(rename = "sol_getVersion")]
pub async fn sol_get_version(source: RpcServices, config: Option<RpcConfig>) -> RpcResult<RpcVersionInfo> {
    let client = rpc_client(source, config);
    client.get_version().await
}

///
/// Returns the account info and associated stake for all the voting accounts in the current bank.
///
#[update(name = "sol_getVoteAccounts")]
#[candid_method(rename = "sol_getVoteAccounts")]
pub async fn sol_get_vote_accounts(
    source: RpcServices,
    config: Option<RpcConfig>,
    params: RpcGetVoteAccountsConfig,
) -> RpcResult<RpcVoteAccountStatus> {
    let client = rpc_client(source, config);
    client.get_vote_accounts(params).await
}

///
/// Requests an airdrop of lamports to a Pubkey.
///
#[update(name = "sol_requestAirdrop")]
#[candid_method(rename = "sol_requestAirdrop")]
pub async fn sol_request_airdrop(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    lamports: u64,
) -> RpcResult<String> {
    let client = rpc_client(source, config);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let signature = client.request_airdrop(&pubkey, lamports).await?;
    Ok(signature)
}

///
/// Submits a signed transaction to the cluster for processing.
/// Use `sol_getSignatureStatuses` to ensure a transaction is processed and confirmed.
///
#[update(name = "sol_sendTransaction")]
#[candid_method(rename = "sol_sendTransaction")]
pub async fn send_transaction(
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

///
/// Retrieves transaction logs for a given public key.
///
/// This asynchronous function connects to the specified RPC `provider`, fetches transaction
/// signatures associated with the provided `pubkey`, and then retrieves detailed transaction
/// data based on those signatures.
///
#[update(name = "sol_getLogs")]
#[candid_method(rename = "sol_getLogs")]
pub async fn sol_get_logs(
    source: RpcServices,
    config: Option<RpcConfig>,
    pubkey: String,
    params: Option<RpcSignaturesForAddressConfig>,
) -> RpcResult<HashMap<String, RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>>> {
    let client = rpc_client(source, config);
    let params = params.unwrap_or_default();
    let commitment = params.commitment;

    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let signatures = client.get_signatures_for_address(&pubkey, params).await?;

    client
        .get_transactions(
            signatures.iter().map(|s| s.signature.as_str()).collect::<Vec<_>>(),
            Some(RpcTransactionConfig {
                commitment,
                ..RpcTransactionConfig::default()
            }),
        )
        .await
}

///
/// Sends a JSON-RPC request to a specified Solana node provider,
/// supporting custom RPC methods.
///
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

///
/// Calculates the cost of an RPC request.
///
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

#[update(guard = "require_manage_or_controller")]
#[candid_method]
fn authorize(principal: Principal, auth: Auth) -> bool {
    do_authorize(principal, auth)
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
    read_metrics(|m| m.clone())
}

/// Cleans up the HTTP response headers to make them deterministic.
///
/// # Arguments
///
/// * `args` - Transformation arguments containing the HTTP response.
///
#[query(hidden = true)]
fn __transform_json_rpc(mut args: TransformArgs) -> HttpResponse {
    // The response header contains non-deterministic fields that make it impossible to reach consensus!
    // Errors seem deterministic and do not contain data that can break consensus.
    // Clear non-deterministic fields from the response headers.
    args.response.headers.clear();
    args.response
}

#[ic_cdk::init]
fn init(args: InitArgs) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(args.into());
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade(_args: InitArgs) {
    // if let Some(v) = args.rpc_url {
    //     mutate_state(|s| s.rpc_url = v);
    // }
}

ic_cdk::export_candid!();

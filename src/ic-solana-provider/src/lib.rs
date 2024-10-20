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
    response::{RpcBlockCommitment, RpcBlockProduction, RpcConfirmedTransactionStatusWithSignature},
    rpc_client::{RpcError, RpcResult},
    types::{
        Account, CandidValue, CommitmentConfig, EncodedConfirmedTransactionWithStatusMeta, EpochInfo, EpochSchedule,
        Pubkey, RpcAccountInfoConfig, RpcBlockConfig, RpcContextConfig, RpcSendTransactionConfig,
        RpcSignatureStatusConfig, RpcSignaturesForAddressConfig, RpcTokenAccountsFilter, RpcTransactionConfig,
        Signature, Slot, TaggedEncodedConfirmedTransactionWithStatusMeta, TaggedRpcBlockProductionConfig,
        TaggedRpcKeyedAccount, TaggedRpcTokenAccountBalance, TaggedUiConfirmedBlock, Transaction, TransactionStatus,
        UiTokenAmount,
    },
};
use ic_solana_common::metrics::{encode_metrics, read_metrics, Metrics};
use serde_json::json;
use state::{read_state, InitArgs};

use crate::{
    auth::{do_authorize, do_deauthorize, require_manage_or_controller, require_register_provider, Auth},
    constants::NODES_IN_SUBNET,
    http::{get_http_request_cost, rpc_client, serve_logs, serve_metrics},
    providers::{do_register_provider, do_unregister_provider, do_update_provider},
    state::STATE,
    types::{RegisterProviderArgs, UpdateProviderArgs},
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
    provider: String,
    pubkey: String,
    config: Option<RpcAccountInfoConfig>,
) -> RpcResult<Option<Account>> {
    let client = rpc_client(&provider);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let account_info = client
        .get_account_info(&pubkey, config.unwrap_or_default(), None)
        .await?;
    Ok(account_info)
}

///
/// Returns the lamport balance of the account of provided Pubkey.
///
#[update(name = "sol_getBalance")]
#[candid_method(rename = "sol_getBalance")]
pub async fn sol_get_balance(provider: String, pubkey: String, config: Option<RpcContextConfig>) -> RpcResult<u64> {
    let client = rpc_client(&provider);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let balance = client.get_balance(&pubkey, config.unwrap_or_default()).await?;
    Ok(balance)
}

///
/// Returns identity and transaction information about a confirmed block in the ledger.
///
#[update(name = "sol_getBlock")]
#[candid_method(rename = "sol_getBlock")]
pub async fn sol_get_block(
    provider: String,
    slot: Slot,
    config: Option<RpcBlockConfig>,
    max_response_bytes: Option<u64>,
) -> RpcResult<TaggedUiConfirmedBlock> {
    let client = rpc_client(&provider);
    let block = client
        .get_block(slot, config.unwrap_or_default(), max_response_bytes)
        .await?;
    Ok(block.into())
}

///
/// Returns commitment for a particular block.
///
#[update(name = "sol_getBlockCommitment")]
#[candid_method(rename = "sol_getBlockCommitment")]
pub async fn sol_get_block_commitment(provider: String, slot: Slot) -> RpcResult<RpcBlockCommitment> {
    let client = rpc_client(&provider);
    client.get_block_commitment(slot).await
}

///
/// Returns the current block height of the node.
///
#[update(name = "sol_getBlockHeight")]
#[candid_method(rename = "sol_getBlockHeight")]
pub async fn sol_get_block_height(provider: String, config: Option<RpcContextConfig>) -> RpcResult<u64> {
    let client = rpc_client(&provider);
    client.get_block_height(config).await
}

///
/// Returns recent block production information from the current or previous epoch.
///
#[update(name = "sol_getBlockProduction")]
#[candid_method(rename = "sol_getBlockProduction")]
pub async fn sol_get_block_production(
    provider: String,
    config: TaggedRpcBlockProductionConfig,
) -> RpcResult<RpcBlockProduction> {
    let client = rpc_client(&provider);
    client.get_block_production(config.into()).await
}

///
/// Returns the estimated production time of a block.
///
#[update(name = "sol_getBlockTime")]
#[candid_method(rename = "sol_getBlockTime")]
pub async fn sol_get_block_time(provider: String, slot: Slot) -> RpcResult<i64> {
    let client = rpc_client(&provider);
    client.get_block_time(slot).await
}

///
/// Returns a list of confirmed blocks between two slots
///
#[update(name = "sol_getBlocks")]
#[candid_method(rename = "sol_getBlocks")]
pub async fn sol_get_blocks(
    provider: String,
    start_slot: Slot,
    last_slot: Option<Slot>,
    config: Option<CommitmentConfig>,
) -> RpcResult<Vec<u64>> {
    let client = rpc_client(&provider);
    client.get_blocks(start_slot, last_slot, config).await
}

///
/// Returns information about the current epoch.
///
#[update(name = "sol_getEpochInfo")]
#[candid_method(rename = "sol_getEpochInfo")]
pub async fn sol_get_epoch_info(provider: String, config: Option<RpcContextConfig>) -> RpcResult<EpochInfo> {
    let client = rpc_client(&provider);
    client.get_epoch_info(config).await
}

///
/// Returns the epoch schedule information from this cluster's genesis config.
///
#[update(name = "sol_getEpochSchedule")]
#[candid_method(rename = "sol_getEpochSchedule")]
pub async fn sol_get_epoch_schedule(provider: String) -> RpcResult<EpochSchedule> {
    let client = rpc_client(&provider);
    client.get_epoch_schedule().await
}

///
/// Returns signatures for confirmed transactions that
/// include the given address in their accountKeys list.
///
#[update(name = "sol_getSignaturesForAddress")]
#[candid_method(rename = "sol_getSignaturesForAddress")]
pub async fn sol_get_signatures_for_address(
    provider: String,
    pubkey: String,
    config: RpcSignaturesForAddressConfig,
) -> RpcResult<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    let client = rpc_client(&provider);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let result = client.get_signatures_for_address(&pubkey, config).await?;
    Ok(result)
}

///
/// Returns the token balance of an SPL Token account.
///
#[update(name = "sol_getTokenAccountBalance")]
#[candid_method(rename = "sol_getTokenAccountBalance")]
pub async fn sol_get_token_account_balance(
    provider: String,
    pubkey: String,
    config: Option<CommitmentConfig>,
) -> RpcResult<UiTokenAmount> {
    let client = rpc_client(&provider);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    client.get_token_account_balance(&pubkey, config).await
}

///
/// Returns all SPL Token accounts by approved Delegate.
///
#[update(name = "sol_getTokenAccountsByDelegate")]
#[candid_method(rename = "sol_getTokenAccountsByDelegate")]
pub async fn sol_get_token_accounts_by_delegate(
    provider: String,
    pubkey: String,
    filter: RpcTokenAccountsFilter,
    config: Option<RpcAccountInfoConfig>,
    max_response_bytes: Option<u64>,
) -> RpcResult<Vec<TaggedRpcKeyedAccount>> {
    let client = rpc_client(&provider);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let accounts = client
        .get_token_accounts_by_delegate(&pubkey, filter, config, max_response_bytes)
        .await?;

    Ok(accounts.into_iter().map(Into::into).collect())
}

///
/// Returns all SPL Token accounts by token owner.
///
#[update(name = "sol_getTokenAccountsByOwner")]
#[candid_method(rename = "sol_getTokenAccountsByOwner")]
pub async fn sol_get_token_accounts_by_owner(
    provider: String,
    pubkey: String,
    filter: RpcTokenAccountsFilter,
    config: Option<RpcAccountInfoConfig>,
    max_response_bytes: Option<u64>,
) -> RpcResult<Vec<TaggedRpcKeyedAccount>> {
    let client = rpc_client(&provider);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let accounts = client
        .get_token_accounts_by_owner(&pubkey, filter, config, max_response_bytes)
        .await?;

    Ok(accounts.into_iter().map(Into::into).collect())
}

///
/// Returns the 20 largest accounts of a particular SPL Token type.
///
#[update(name = "sol_getTokenLargestAccounts")]
#[candid_method(rename = "sol_getTokenLargestAccounts")]
pub async fn sol_get_token_largest_accounts(
    provider: String,
    mint: String,
    config: Option<CommitmentConfig>,
    max_response_bytes: Option<u64>,
) -> RpcResult<Vec<TaggedRpcTokenAccountBalance>> {
    let client = rpc_client(&provider);
    let mint = Pubkey::from_str(&mint).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let accounts = client
        .get_token_largest_accounts(&mint, config, max_response_bytes)
        .await?;

    Ok(accounts.into_iter().map(Into::into).collect())
}

///
/// Returns the total supply of an SPL Token type.
///
#[update(name = "sol_getTokenSupply")]
#[candid_method(rename = "sol_getTokenSupply")]
pub async fn sol_get_token_supply(
    provider: String,
    mint: String,
    config: Option<CommitmentConfig>,
    max_response_bytes: Option<u64>,
) -> RpcResult<UiTokenAmount> {
    let client = rpc_client(&provider);
    let mint = Pubkey::from_str(&mint).map_err(|e| RpcError::ParseError(e.to_string()))?;
    client.get_token_supply(&mint, config, max_response_bytes).await
}

///
/// Returns the latest blockhash.
///
#[update(name = "sol_getLatestBlockhash")]
#[candid_method(rename = "sol_getLatestBlockhash")]
pub async fn sol_get_latest_blockhash(provider: String, config: Option<RpcContextConfig>) -> RpcResult<String> {
    let client = rpc_client(&provider);
    let blockhash = client.get_latest_blockhash(config).await?;
    Ok(blockhash.to_string())
}

///
/// Returns the statuses of a list of signatures.
/// Each signature must be a txid, the first signature of a transaction.
///
#[update(name = "sol_getSignatureStatuses")]
#[candid_method(rename = "sol_getSignatureStatuses")]
pub async fn sol_get_signature_statuses(
    provider: String,
    signatures: Vec<String>,
    config: Option<RpcSignatureStatusConfig>,
) -> RpcResult<Vec<Option<TransactionStatus>>> {
    let client = rpc_client(&provider);

    let signatures: Vec<Signature> = signatures
        .into_iter()
        .map(|s| Signature::from_str(&s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| RpcError::ParseError(e.to_string()))?;

    client.get_signature_statuses(&signatures, config).await
}

///
/// Returns transaction details for a confirmed transaction.
///
#[update(name = "sol_getTransaction")]
#[candid_method(rename = "sol_getTransaction")]
pub async fn sol_get_transaction(
    provider: String,
    signature: String,
    config: Option<RpcTransactionConfig>,
    max_response_bytes: Option<u64>,
) -> RpcResult<TaggedEncodedConfirmedTransactionWithStatusMeta> {
    let client = rpc_client(&provider);
    let signature = Signature::from_str(&signature).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let response = client.get_transaction(&signature, config, max_response_bytes).await?;
    Ok(response.into())
}

///
/// Requests an airdrop of lamports to a Pubkey.
///
#[update(name = "sol_requestAirdrop")]
#[candid_method(rename = "sol_requestAirdrop")]
pub async fn sol_request_airdrop(provider: String, pubkey: String, lamports: u64) -> RpcResult<String> {
    let client = rpc_client(&provider);
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
    provider: String,
    raw_signed_transaction: String,
    config: Option<RpcSendTransactionConfig>,
) -> RpcResult<String> {
    let client = rpc_client(&provider);
    let tx = Transaction::from_str(&raw_signed_transaction).expect("Invalid transaction");
    let signature = client.send_transaction(tx, config.unwrap_or_default()).await?;
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
    provider: String,
    pubkey: String,
    config: Option<RpcSignaturesForAddressConfig>,
    max_response_bytes: Option<u64>,
) -> RpcResult<HashMap<String, RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>>> {
    let client = rpc_client(&provider);
    let config = config.unwrap_or_default();
    let commitment = config.commitment;

    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let signatures = client.get_signatures_for_address(&pubkey, config).await?;

    client
        .get_transactions(
            signatures.iter().map(|s| s.signature.as_str()).collect::<Vec<_>>(),
            RpcTransactionConfig {
                encoding: None,
                commitment,
                max_supported_transaction_version: None,
            },
            max_response_bytes,
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
    provider: String,
    method: String,
    params: CandidValue,
    max_response_bytes: u64,
) -> RpcResult<String> {
    let client = rpc_client(&provider);
    let payload = json!({
        "jsonrpc": "2.0",
        "id": client.next_request_id(),
        "method": &method,
        "params": params
    });
    let res = client.call(&payload, max_response_bytes).await?;
    String::from_utf8(res).map_err(|e| RpcError::ParseError(e.to_string()))
}

///
/// Calculates the cost of an RPC request.
///
#[query(name = "requestCost")]
#[candid_method(query, rename = "requestCost")]
pub fn request_cost(json_rpc_payload: String, max_response_bytes: u64) -> u128 {
    get_http_request_cost(json_rpc_payload.len() as u64, max_response_bytes)
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

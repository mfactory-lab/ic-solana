use {
    crate::{
        auth::{
            do_authorize, do_deauthorize, require_manage_or_controller, require_register_provider,
            Auth,
        },
        constants::NODES_IN_SUBNET,
        http::{get_http_request_cost, rpc_client, serve_logs, serve_metrics},
        providers::{do_register_provider, do_unregister_provider, do_update_provider},
        state::STATE,
        types::{RegisterProviderArgs, SendTransactionRequest, UpdateProviderArgs},
        utils::validate_caller_not_anonymous,
    },
    candid::{candid_method, Principal},
    eddsa_api::{eddsa_public_key, sign_with_eddsa},
    ic_canisters_http_types::{
        HttpRequest as AssetHttpRequest, HttpResponse as AssetHttpResponse, HttpResponseBuilder,
    },
    ic_cdk::{
        api::management_canister::http_request::{HttpResponse, TransformArgs},
        query, update,
    },
    ic_solana::{
        response::{RpcBlockProduction, RpcConfirmedTransactionStatusWithSignature},
        rpc_client::{RpcError, RpcResult},
        types::{
            Account, BlockHash, CandidValue, CommitmentConfig,
            EncodedConfirmedTransactionWithStatusMeta, Instruction, Message, Pubkey,
            RpcAccountInfoConfig, RpcContextConfig, RpcSendTransactionConfig,
            RpcSignatureStatusConfig, RpcSignaturesForAddressConfig, RpcTransactionConfig,
            Signature, Slot, TaggedEncodedConfirmedTransactionWithStatusMeta,
            TaggedRpcBlockProductionConfig, TaggedRpcKeyedAccount, TaggedRpcTokenAccountBalance,
            TaggedUiConfirmedBlock, TokenAccountsFilter, Transaction, TransactionDetails,
            TransactionStatus, UiAccountEncoding, UiTokenAmount, UiTransactionEncoding,
        },
    },
    ic_solana_common::metrics::{encode_metrics, read_metrics, Metrics},
    serde_bytes::ByteBuf,
    serde_json::json,
    state::{read_state, InitArgs},
    std::{collections::HashMap, str::FromStr},
};

pub mod auth;
mod constants;
pub mod eddsa_api;
mod http;
mod memory;
mod providers;
pub mod state;
pub mod types;
mod utils;

///
/// Returns the public key of the Solana wallet for the caller.
///
#[update]
#[candid_method]
pub async fn sol_address() -> String {
    let caller = validate_caller_not_anonymous();
    let key_name = read_state(|s| s.schnorr_key.clone());
    let derived_path = vec![ByteBuf::from(caller.as_slice())];
    let pk = eddsa_public_key(key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice())
        .expect("Invalid public key")
        .to_string()
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

    let signatures = client
        .get_signatures_for_address(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            config,
        )
        .await?;

    client
        .get_transactions(
            signatures
                .iter()
                .map(|s| s.signature.as_str())
                .collect::<Vec<_>>(),
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
    let result = client
        .get_signatures_for_address(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            config,
        )
        .await?;
    Ok(result)
}

///
/// Requests an airdrop of lamports to a Pubkey.
///
#[update(name = "sol_requestAirdrop")]
#[candid_method(rename = "sol_requestAirdrop")]
pub async fn sol_request_airdrop(
    provider: String,
    pubkey: String,
    lamports: u64,
) -> RpcResult<String> {
    let client = rpc_client(&provider);
    let signature = client
        .request_airdrop(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            lamports,
        )
        .await?;
    Ok(signature)
}

///
/// Returns the lamport balance of the account of provided Pubkey.
///
#[update(name = "sol_getBalance")]
#[candid_method(rename = "sol_getBalance")]
pub async fn sol_get_balance(provider: String, pubkey: String) -> RpcResult<u64> {
    let client = rpc_client(&provider);
    let balance = client
        .get_balance(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            RpcContextConfig::default(),
        )
        .await?;
    Ok(balance)
}

///
/// Returns identity and transaction information about a confirmed block in the ledger
///
#[update(name = "sol_getBlock")]
#[candid_method(rename = "sol_getBlock")]
pub async fn sol_get_block(
    provider: String,
    slot: u64,
    transaction_details: TransactionDetails,
    max_response_bytes: Option<u64>,
) -> RpcResult<TaggedUiConfirmedBlock> {
    let client = rpc_client(&provider);
    let block = client
        .get_block(
            slot,
            UiTransactionEncoding::Json,
            transaction_details,
            max_response_bytes,
        )
        .await?;
    Ok(block.into())
}

///
/// Returns a list of confirmed blocks between two slots
///
#[update(name = "sol_getBlocks")]
#[candid_method(rename = "sol_getBlocks")]
pub async fn sol_get_blocks(
    provider: String,
    start_slot: u64,
    last_slot: Option<u64>,
) -> RpcResult<Vec<u64>> {
    let client = rpc_client(&provider);
    let blocks = client.get_blocks(start_slot, last_slot).await?;

    Ok(blocks)
}

///
/// Returns the current block height of the node
///
#[update(name = "sol_getBlockHeight")]
#[candid_method(rename = "sol_getBlockHeight")]
pub async fn sol_get_block_height(provider: String) -> RpcResult<u64> {
    let client = rpc_client(&provider);
    let commitment = None;

    let height = client.get_block_height(commitment).await?;
    Ok(height)
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
    let block_production = client.get_block_production(config.into()).await?;

    Ok(block_production)
}

///
/// Returns the token balance of an SPL Token account.
///
#[update(name = "sol_getTokenBalance")]
#[candid_method(rename = "sol_getTokenBalance")]
pub async fn sol_get_token_balance(provider: String, pubkey: String) -> RpcResult<UiTokenAmount> {
    let client = rpc_client(&provider);
    let commitment = None;
    let balance = client
        .get_token_account_balance(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            commitment,
        )
        .await?;
    Ok(balance)
}

///
/// Returns all SPL Token accounts by token owner.
///
#[update(name = "sol_getTokenAccountsByOwner")]
#[candid_method(rename = "sol_getTokenAccountsByOwner")]
pub async fn sol_get_token_accounts_by_owner(
    provider: String,
    pubkey: String,
    token_accounts_filter: TokenAccountsFilter,
    max_response_bytes: Option<u64>,
) -> RpcResult<Vec<TaggedRpcKeyedAccount>> {
    let client = rpc_client(&provider);
    let accounts = client
        .get_token_accounts_by_owner(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            token_accounts_filter,
            max_response_bytes,
        )
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
    max_response_bytes: Option<u64>,
) -> RpcResult<Vec<TaggedRpcTokenAccountBalance>> {
    let client = rpc_client(&provider);
    let accounts = client
        .get_token_largest_accounts(
            &Pubkey::from_str(&mint).expect("Invalid public key"),
            max_response_bytes,
        )
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
    max_response_bytes: Option<u64>,
) -> RpcResult<UiTokenAmount> {
    let client = rpc_client(&provider);
    let supply = client
        .get_token_supply(
            &Pubkey::from_str(&mint).expect("Invalid public key"),
            max_response_bytes,
        )
        .await?;

    Ok(supply)
}

///
/// Returns all SPL Token accounts by approved Delegate.
///
#[update(name = "sol_getTokenAccountsByDelegate")]
#[candid_method(rename = "sol_getTokenAccountsByDelegate")]
pub async fn sol_get_token_accounts_by_delegate(
    provider: String,
    pubkey: String,
    token_accounts_filter: TokenAccountsFilter,
    max_response_bytes: Option<u64>,
) -> RpcResult<Vec<TaggedRpcKeyedAccount>> {
    let client = rpc_client(&provider);
    let accounts = client
        .get_token_accounts_by_delegate(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            token_accounts_filter,
            max_response_bytes,
        )
        .await?;

    Ok(accounts.into_iter().map(Into::into).collect())
}

///
/// Returns the latest blockhash.
///
#[update(name = "sol_getLatestBlockhash")]
#[candid_method(rename = "sol_getLatestBlockhash")]
pub async fn sol_get_latest_blockhash(provider: String) -> RpcResult<String> {
    let client = rpc_client(&provider);
    let blockhash = client
        .get_latest_blockhash(RpcContextConfig::default())
        .await?;
    Ok(blockhash.to_string())
}

///
/// Returns all information associated with the account of the provided Pubkey.
///
#[update(name = "sol_getAccountInfo")]
#[candid_method(rename = "sol_getAccountInfo")]
pub async fn sol_get_account_info(provider: String, pubkey: String) -> RpcResult<Option<Account>> {
    let client = rpc_client(&provider);
    let pubkey = Pubkey::from_str(&pubkey).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let account_info = client
        .get_account_info(
            &pubkey,
            RpcAccountInfoConfig {
                // Encoded binary (base58) data should be less than 128 bytes, so use base64 encoding.
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                commitment: None,
                min_context_slot: None,
            },
            None,
        )
        .await?;
    Ok(account_info)
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
) -> RpcResult<Vec<Option<TransactionStatus>>> {
    let client = rpc_client(&provider);

    let signatures: Vec<Signature> = signatures
        .into_iter()
        .map(|s| Signature::from_str(&s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| RpcError::ParseError(e.to_string()))?;

    let response = client
        .get_signature_statuses(
            &signatures,
            RpcSignatureStatusConfig {
                search_transaction_history: false,
            },
        )
        .await?;

    Ok(response)
}

///
/// Returns transaction details for a confirmed transaction.
///
#[update(name = "sol_getTransaction")]
#[candid_method(rename = "sol_getTransaction")]
pub async fn sol_get_transaction(
    provider: String,
    signature: String,
    max_response_bytes: Option<u64>,
) -> RpcResult<TaggedEncodedConfirmedTransactionWithStatusMeta> {
    let client = rpc_client(&provider);
    let signature =
        Signature::from_str(&signature).map_err(|e| RpcError::ParseError(e.to_string()))?;
    let response = client
        .get_transaction(
            &signature,
            RpcTransactionConfig {
                max_supported_transaction_version: Some(0),
                ..RpcTransactionConfig::default()
            },
            max_response_bytes,
        )
        .await?;
    Ok(response.into())
}

///
/// Send a transaction to the network.
///
#[update(name = "sol_sendTransaction")]
#[candid_method(rename = "sol_sendTransaction")]
pub async fn sol_send_transaction(
    provider: String,
    req: SendTransactionRequest,
) -> RpcResult<String> {
    let caller = validate_caller_not_anonymous();
    let client = rpc_client(&provider);

    let recent_blockhash = match req.recent_blockhash {
        Some(r) => BlockHash::from_str(&r).expect("Invalid recent blockhash"),
        None => {
            client
                .get_latest_blockhash(RpcContextConfig::default())
                .await?
        }
    };

    let ixs = &req
        .instructions
        .iter()
        .map(|s| Instruction::from_str(s).unwrap())
        .collect::<Vec<_>>();

    let message = Message::new_with_blockhash(ixs, None, &recent_blockhash);

    let mut tx = Transaction::new_unsigned(message);

    let key_name = read_state(|s| s.schnorr_key.clone());
    let derived_path = vec![ByteBuf::from(caller.as_slice())];

    let signature = sign_with_eddsa(key_name, derived_path, tx.message_data())
        .await
        .try_into()
        .expect("Invalid signature");

    tx.add_signature(0, signature);

    let signature = client
        .send_transaction(tx, RpcSendTransactionConfig::default())
        .await?;

    Ok(signature.to_string())
}

///
/// Submits a signed transaction to the cluster for processing.
///
#[update(name = "sol_sendRawTransaction")]
#[candid_method(rename = "sol_sendRawTransaction")]
pub async fn send_raw_transaction(
    provider: String,
    raw_signed_transaction: String,
) -> RpcResult<String> {
    let client = rpc_client(&provider);

    let tx = Transaction::from_str(&raw_signed_transaction).expect("Invalid transaction");

    let signature = client
        .send_transaction(tx, RpcSendTransactionConfig::default())
        .await?;

    Ok(signature.to_string())
}

///
/// Calls a JSON-RPC method on a Solana node at the specified URL.
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

use crate::types::SendTransactionRequest;
use crate::utils::{rpc_client, validate_caller_not_anonymous};
use eddsa_api::{eddsa_public_key, sign_with_eddsa};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
};
use ic_cdk::{query, update};
use ic_solana::http_request_required_cycles;
use ic_solana::rpc_client::RpcResult;
use ic_solana::types::{
    Account, BlockHash, EncodedConfirmedTransactionWithStatusMeta, Instruction, Message, Pubkey,
    RpcAccountInfoConfig, RpcContextConfig, RpcSendTransactionConfig, RpcTransactionConfig,
    Signature, Transaction, UiAccountEncoding, UiTokenAmount,
};
use serde_bytes::ByteBuf;
use serde_json::json;
use state::{mutate_state, read_state, InitArgs, STATE};
use std::str::FromStr;

mod constants;
pub mod eddsa_api;
pub mod state;
pub mod types;
mod utils;

///
/// Returns the public key of the Solana wallet for the caller.
///
#[update]
pub async fn get_address() -> String {
    let caller = validate_caller_not_anonymous();
    let key_name = read_state(|s| s.schnorr_key_name.clone());
    let derived_path = vec![ByteBuf::from(caller.as_slice())];
    let pk = eddsa_public_key(key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice())
        .expect("Invalid public key")
        .to_string()
}

///
/// Calls a JSON-RPC method on a Solana node at the specified URL.
///
#[update]
pub async fn request(method: String, params: String, max_response_bytes: u64) -> RpcResult<String> {
    let client = rpc_client();
    let payload = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": client.next_request_id(),
        "method": &method,
        "params": params
    }))?;
    client.call(&payload, max_response_bytes).await
}

///
/// Calculates the cost of an RPC request.
///
#[query(name = "requestCost")]
pub fn request_cost(payload: String, max_response_bytes: u64) -> u128 {
    let client = rpc_client();

    let request = CanisterHttpRequestArgument {
        url: client.cluster.url().to_string(),
        max_response_bytes: Some(max_response_bytes),
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
        body: Some(payload.as_bytes().to_vec()),
        transform: None,
    };

    http_request_required_cycles(&request, read_state(|s| s.nodes_in_subnet))
}

///
/// Returns the lamport balance of the account of provided Pubkey.
///
#[update(name = "sol_getBalance")]
pub async fn sol_get_balance(pubkey: String) -> RpcResult<u64> {
    let client = rpc_client();
    let balance = client
        .get_balance(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            RpcContextConfig::default(),
        )
        .await?;
    Ok(balance)
}

///
/// Returns the token balance of an SPL Token account.
///
#[update(name = "sol_getTokenBalance")]
pub async fn sol_get_token_balance(pubkey: String) -> RpcResult<UiTokenAmount> {
    let client = rpc_client();
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
/// Returns the latest blockhash.
///
#[update(name = "sol_latestBlockhash")]
pub async fn sol_get_latest_blockhash() -> RpcResult<String> {
    let client = rpc_client();
    let blockhash = client
        .get_latest_blockhash(RpcContextConfig::default())
        .await?;
    Ok(blockhash.to_string())
}

///
/// Returns all information associated with the account of provided Pubkey.
///
#[update(name = "sol_getAccountInfo")]
pub async fn sol_get_account_info(pubkey: String) -> RpcResult<Option<Account>> {
    let client = rpc_client();
    let account_info = client
        .get_account_info(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
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
/// Returns transaction details for a confirmed transaction.
///
#[update(name = "sol_getTransaction")]
pub async fn sol_get_transaction(
    signature: String,
) -> RpcResult<EncodedConfirmedTransactionWithStatusMeta> {
    let client = rpc_client();
    let signature = Signature::from_str(&signature).expect("Invalid signature");
    let response = client
        .get_transaction(&signature, RpcTransactionConfig::default())
        .await?;
    Ok(response)
}

///
/// Send a transaction to the network.
///
#[update(name = "sol_sendTransaction")]
pub async fn sol_send_transaction(req: SendTransactionRequest) -> RpcResult<String> {
    let caller = validate_caller_not_anonymous();
    let client = rpc_client();

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

    let key_name = read_state(|s| s.schnorr_key_name.clone());
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
pub async fn send_raw_transaction(raw_signed_transaction: String) -> RpcResult<String> {
    let client = rpc_client();

    let tx = Transaction::from_str(&raw_signed_transaction).expect("Invalid transaction");

    let signature = client
        .send_transaction(tx, RpcSendTransactionConfig::default())
        .await?;

    Ok(signature.to_string())
}

/// Cleans up the HTTP response headers to make them deterministic.
///
/// # Arguments
///
/// * `args` - Transformation arguments containing the HTTP response.
///
#[query(hidden = true)]
fn cleanup_response(mut args: TransformArgs) -> HttpResponse {
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
fn post_upgrade(args: InitArgs) {
    if let Some(v) = args.nodes_in_subnet {
        mutate_state(|s| s.nodes_in_subnet = v);
    }
    if let Some(v) = args.rpc_url {
        mutate_state(|s| s.rpc_url = v);
    }
}

ic_cdk::export_candid!();

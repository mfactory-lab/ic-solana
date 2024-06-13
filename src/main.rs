use candid::{CandidType, Principal};
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk::{query, update};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::json;
use solana_rpc::eddsa_api::{eddsa_public_key, sign_with_eddsa};
use solana_rpc::rpc_client::{RpcClient, RpcResult};
use solana_rpc::state::{mutate_state, read_state, InitArgs, State, STATE};
use solana_rpc::types::account::{Account, UiAccountEncoding, UiTokenAmount};
use solana_rpc::types::blockhash::BlockHash;
use solana_rpc::types::config::{RpcAccountInfoConfig, RpcContextConfig, RpcSendTransactionConfig};
use solana_rpc::types::instruction::Instruction;
use solana_rpc::types::message::Message;
use solana_rpc::types::pubkey::Pubkey;
use solana_rpc::types::transaction::{EncodedConfirmedTransactionWithStatusMeta, Transaction};
use solana_rpc::utils::get_http_request_cost;
use std::str::FromStr;

fn validate_caller_not_anonymous() -> Principal {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        panic!("Anonymous principal not allowed to make calls.")
    }
    caller
}

///
/// Returns the public key of the Solana wallet for the caller.
///
#[update]
pub async fn get_sol_address() -> String {
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
    let client = read_state(RpcClient::from_state);
    let payload = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": mutate_state(State::next_request_id),
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
    let client = read_state(RpcClient::from_state);
    get_http_request_cost(
        client.cluster.url(),
        payload.len() as u64,
        max_response_bytes,
    )
}

///
/// Returns the lamport balance of the account of provided Pubkey.
///
#[update(name = "sol_getBalance")]
pub async fn sol_get_balance(pubkey: String) -> RpcResult<u64> {
    let client = read_state(RpcClient::from_state);
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
    let client = read_state(RpcClient::from_state);
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
/// Returns all information associated with the account of provided Pubkey.
///
#[update(name = "sol_getAccountInfo")]
pub async fn sol_get_account_info(pubkey: String) -> RpcResult<Option<Account>> {
    let client = read_state(RpcClient::from_state);
    let balance = client
        .get_account_info(
            &Pubkey::from_str(&pubkey).expect("Invalid public key"),
            RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base58),
                data_slice: None,
                commitment: None,
                min_context_slot: None,
            },
            None,
        )
        .await?;
    Ok(balance)
}

// ///
// /// Returns transaction details for a confirmed transaction.
// ///
// #[update(name = "sol_getTransaction")]
// #[candid_method(rename = "sol_getTransaction")]
// pub async fn sol_get_transaction(
//     signature: String,
// ) -> RpcResult<EncodedConfirmedTransactionWithStatusMeta> {
//     let client = read_state(RpcClient::from_state);
//     let signature = Signature::from_str(&signature).expect("Invalid signature");
//     let response = client.get_transaction(&signature, None).await?;
//     Ok(response)
// }

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct SendTransactionRequest {
    instructions: Vec<Instruction>,
    recent_blockhash: Option<String>,
}

///
/// Send a transaction to the network.
///
#[update(name = "sol_sendTransaction")]
pub async fn sol_send_transaction(req: SendTransactionRequest) -> RpcResult<String> {
    let caller = validate_caller_not_anonymous();
    let client = read_state(RpcClient::from_state);

    let recent_blockhash = match req.recent_blockhash {
        Some(r) => BlockHash::from_str(&r).expect("Invalid recent blockhash"),
        None => {
            client
                .get_latest_blockhash(RpcContextConfig::default())
                .await?
        }
    };

    let message = Message::new_with_blockhash(&req.instructions, None, &recent_blockhash);

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
    let client = read_state(RpcClient::from_state);

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

fn main() {}
ic_cdk::export_candid!();

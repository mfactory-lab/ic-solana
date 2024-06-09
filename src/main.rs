use candid::candid_method;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk::{query, update};
use ic_cdk_macros::{init, post_upgrade};
use ic_crypto_ed25519::PublicKey;
use serde_json::json;
use solana_rpc::cluster::Cluster;
use solana_rpc::config::RpcAccountInfoConfig;
use solana_rpc::constants::NODES_IN_FIDUCIARY_SUBNET;
use solana_rpc::rpc_client::{RpcClient, RpcResult};
use solana_rpc::state::{mutate_state, read_state, State, STATE};
use solana_rpc::types::account::{Account, UiAccountEncoding};
use solana_rpc::types::transaction::EncodedConfirmedTransactionWithStatusMeta;
use solana_rpc::types::InitArgs;
use solana_rpc::utils::get_http_request_cost;

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

/// Verify an EdDSA signature (message signed by Solana wallet).
#[query]
#[candid_method]
pub fn verify_eddsa(pubkey: String, msg: String, signature: String) -> bool {
    let pubkey = PublicKey::deserialize_raw(pubkey.as_bytes()).expect("invalid public key");
    pubkey
        .verify_signature(msg.as_bytes(), signature.as_bytes())
        .is_ok()
}

/// Calls a JSON-RPC method on a Solana node at the specified URL.
#[update]
#[candid_method]
pub async fn request(method: String, params: String, max_response_bytes: u64) -> RpcResult<String> {
    let client = read_state(RpcClient::from_state);

    let payload = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": mutate_state(State::next_request_id),
        "method": &method,
        "params": params
    }))?;

    let res = client.call(&payload, max_response_bytes).await?;

    Ok(res)
}

/// Calculates the cost of an RPC request.
#[query(name = "requestCost")]
#[candid_method(query, rename = "requestCost")]
pub fn request_cost(payload: String, max_response_bytes: u64) -> u128 {
    let client = read_state(RpcClient::from_state);
    get_http_request_cost(
        client.cluster.url(),
        payload.len() as u64,
        max_response_bytes,
    )
}

#[update(name = "sol_getBalance")]
#[candid_method(rename = "sol_getBalance")]
pub async fn sol_get_balance(address: String) -> RpcResult<u64> {
    let client = read_state(RpcClient::from_state);
    let balance = client.get_balance(&address, None).await?;
    Ok(balance)
}

#[update(name = "sol_getAccountInfo")]
#[candid_method(rename = "sol_getAccountInfo")]
pub async fn sol_get_account_info(address: String) -> RpcResult<Option<Account>> {
    let client = read_state(RpcClient::from_state);
    let balance = client
        .get_account_info(
            &address,
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

#[update(name = "sol_getTransaction")]
#[candid_method(rename = "sol_getTransaction")]
pub async fn sol_get_transaction(
    signature: String,
) -> RpcResult<EncodedConfirmedTransactionWithStatusMeta> {
    let client = read_state(RpcClient::from_state);
    let response = client.get_transaction(&signature, None, 10 * 1024).await?;
    Ok(response)
}

// #[update(name = "sendRawTransaction")]
// #[candid_method(rename = "sendRawTransaction")]
// pub async fn _send_raw_transaction(
//     source: RpcServices,
//     config: Option<RpcConfig>,
//     raw_signed_transaction_hex: String,
// ) -> MultiRpcResult<candid_types::SendRawTransactionStatus> {
//     match CandidRpcClient::new(source, config) {
//         Ok(source) => {
//             source
//                 .eth_send_raw_transaction(raw_signed_transaction_hex)
//                 .await
//         }
//         Err(err) => Err(err).into(),
//     }
// }

// #[update]
// async fn send_tx() {
//     let user = caller();
//
//     debug_println_caller("send_tx");
//
//     // api::print(format!("| balance start: {}", api::canister_balance()));
//
//     // let kp = get_solana_keypair().await;
//     //
//     // api::print(format!("solana keypair: {:?}", kp.sk.as_ref()));
//     // api::print(format!("solana keypair: {}", hex::encode(kp.pk.as_ref())));
//
//     // api::print(format!("| balance end: {}", api::canister_balance()));
//
//     // let program_id = pubkey!("ALBs64hsiHgdg53mvd4bcvNZLfDRhctSVaP7PwAPpsZL");
//     //
//     // let ix_data = 0u8;
//     // let ix = Instruction::new_with_borsh(program_id, &ix_data, vec![]);
//     //
//     // let msg = Message::new(&[ix], None);
//     //
//     // api::print(format!("msg: {:?}", msg.serialize()));
//     //
//     // let blockhash = Hash::new(&[]);
//     // let tx = Transaction::new(&[&payer], msg, blockhash);
//
//     // TODO: sign transaction
//
//     // let res = call_solana("getLatestBlockhash", Value::Null)
//     //     .await
//     //     .unwrap();
//     //
//     // let data: Value = serde_json::from_slice(&res.0.body).unwrap();
//
//     // api::print(format!("response: {:?}", data));
// }

/// Cleans up the HTTP response headers to make them deterministic.
///
/// # Arguments
///
/// * `args` - Transformation arguments containing the HTTP response.
#[query(hidden = true)]
fn cleanup_response(mut args: TransformArgs) -> HttpResponse {
    // The response header contains non-deterministic fields that make it impossible to reach consensus!
    // Errors seem deterministic and do not contain data that can break consensus.
    // Clear non-deterministic fields from the response headers.
    args.response.headers.clear();
    args.response
}

#[init]
fn init(args: InitArgs) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(State {
            nodes_in_subnet: args.nodes_in_subnet.unwrap_or(NODES_IN_FIDUCIARY_SUBNET),
            rpc_url: args.rpc_url.unwrap_or(Cluster::Devnet.to_string()),
            http_request_counter: 0,
        })
    });
}

#[post_upgrade]
fn post_upgrade(args: InitArgs) {
    if let Some(v) = args.nodes_in_subnet {
        mutate_state(|s| s.nodes_in_subnet = v);
    }
    if let Some(v) = args.rpc_url {
        mutate_state(|s| s.rpc_url = v);
    }
}

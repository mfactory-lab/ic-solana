mod types;

use ic_cdk::api;
use ic_cdk::api::call::CallResult;
use ic_cdk::api::caller as caller_api;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
};
use ic_cdk::export::Principal;
use ic_cdk_macros::*;
use std::str::FromStr;

use ic_crypto_internal_bls12_381_vetkd::{DerivedPublicKey, EncryptedKey, TransportSecretKey};

use serde_json::{self, json, Value};

use types::{
    CanisterId, VetKDCurve, VetKDEncryptedKeyReply, VetKDEncryptedKeyRequest, VetKDKeyId,
    VetKDPublicKeyReply, VetKDPublicKeyRequest,
};

const VETKD_SYSTEM_API_CANISTER_ID: &str = "nqbbm-nqaaa-aaaak-ae5sa-cai";

/// Transport secret key used for internal `VetKD` calls.
const TRANSPORT_SK_HEX: &str = "718c36cd1dcf3501fd04bbe24c3bb9eedfd066d2420e794dd9342cf71d04176f";

lazy_static::lazy_static! {
    static ref TRANSPORT_SK: TransportSecretKey = TransportSecretKey::deserialize(
        &hex::decode(TRANSPORT_SK_HEX).expect("failed to hex-decode")
    ).expect("failed to deserialize Scalar");
    // static ref TRANSPORT_PK: G2Affine = G2Affine::from(G2Affine::generator() * &*TRANSPORT_SK);
}

/// Thus, we use the ic_cdk::api::caller() method inside this wrapper function.
/// The wrapper prevents the use of the anonymous identity. Forbidding anonymous
/// interactions is the recommended default behavior for IC canisters.
fn caller() -> Principal {
    let caller = caller_api();
    // The anonymous principal is not allowed to interact with the
    // encrypted notes canister.
    if caller == Principal::anonymous() {
        panic!("Anonymous principal not allowed to make calls.")
    }
    caller
}

#[init]
fn init() {}

/// Reflects the [caller]'s identity by returning (a future of) its principal.
/// Useful for debugging.
#[query]
fn whoami() -> String {
    caller_api().to_string()
}

/// Makes a JSON-RPC call to the Solana blockchain using the specified method and parameters.
///
/// # Arguments
///
/// * `method` - A string representing the JSON-RPC method to be called.
/// * `params` - A `serde_json::Value` representing the parameters to be passed in the JSON-RPC call.
///
/// # Returns
///
/// A `Result` containing a tuple with the HTTP response wrapped in an `HttpResponse` enum.
///
async fn call_solana(method: &str, params: Value) -> CallResult<(HttpResponse,)> {
    let cluster = "https://api.devnet.solana.com";

    let body = json!({
      "id": 1,
      "jsonrpc": "2.0",
      "method": method,
      "params": params
    });

    let request = CanisterHttpRequestArgument {
        url: cluster.to_string(),
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
        body: Some(body.to_string().into_bytes()),
        max_response_bytes: None,
        transform: None,
    };

    http_request(request).await
}

/// Retrieves a Solana keypair for the current user's principal using the VetKD system.
///
/// This function performs the following steps:
/// 1. Calls the `vetkd_encrypted_key` method to obtain an encrypted key.
/// 2. Deserializes the encrypted key and calls the `vetkd_public_key` method to get the derived public key.
/// 3. Decrypts the encrypted key using the derived public key to obtain the final Solana keypair.
///
/// # Returns
///
/// An `ed25519_compact::KeyPair` representing the Solana keypair for the user's principal.
///
/// # Panics
///
/// This function panics if any of the asynchronous calls to the VetKD system or key-related operations fail.
/// Ensure that the VetKD system is reachable and responsive.
///
async fn get_solana_keypair() -> ed25519_compact::KeyPair {
    let user_principal = caller();
    let derivation_path = vec![b"solana_key".to_vec()];

    let (response,): (VetKDEncryptedKeyReply,) = api::call::call(
        vetkd_system_api_canister_id(),
        "vetkd_encrypted_key",
        (VetKDEncryptedKeyRequest {
            derivation_id: user_principal.as_slice().to_vec(),
            public_key_derivation_path: derivation_path.clone(),
            key_id: bls12_381_test_key_1(),
            encryption_public_key: TRANSPORT_SK.public_key().serialize().into(),
        },),
    )
    .await
    .expect("call to vetkd_encrypted_key failed");

    let ek = EncryptedKey::deserialize(response.encrypted_key.try_into().unwrap())
        .expect("failed to obtain encrypted key");

    let (response,): (VetKDPublicKeyReply,) = api::call::call(
        vetkd_system_api_canister_id(),
        "vetkd_public_key",
        (VetKDPublicKeyRequest {
            canister_id: None,
            derivation_path,
            key_id: bls12_381_test_key_1(),
        },),
    )
    .await
    .expect("call to vetkd_public_key failed");

    let dpk = DerivedPublicKey::deserialize(&response.public_key).unwrap();
    let key = TRANSPORT_SK
        .decrypt_and_hash(&ek, &dpk, user_principal.as_ref(), 32, &[])
        .expect("failed to decrypt symmetric key");

    ed25519_compact::KeyPair::from_seed(
        ed25519_compact::Seed::from_slice(&key).expect("failed to calculate ed25519 keypair seed"),
    )
}

#[ic_cdk::update]
async fn send_tx() {
    let user = caller();

    debug_println_caller("send_tx");

    // api::print(format!("| balance start: {}", api::canister_balance()));

    let kp = get_solana_keypair().await;

    api::print(format!("solana keypair: {:?}", kp.sk.as_ref()));
    // api::print(format!("solana keypair: {}", hex::encode(kp.pk.as_ref())));

    // api::print(format!("| balance end: {}", api::canister_balance()));

    // let program_id = pubkey!("ALBs64hsiHgdg53mvd4bcvNZLfDRhctSVaP7PwAPpsZL");
    //
    // let ix_data = 0u8;
    // let ix = Instruction::new_with_borsh(program_id, &ix_data, vec![]);
    //
    // let msg = Message::new(&[ix], None);
    //
    // api::print(format!("msg: {:?}", msg.serialize()));
    //
    // let blockhash = Hash::new(&[]);
    // let tx = Transaction::new(&[&payer], msg, blockhash);

    // TODO: sign transaction

    let res = call_solana("getLatestBlockhash", Value::Null)
        .await
        .unwrap();

    let data: Value = serde_json::from_slice(&res.0.body).unwrap();

    api::print(format!("response: {:?}", data));
}

/// Hooks in these macros will produce a `function already defined` error
/// if they share the same name as the underlying function.

#[pre_upgrade]
/// The pre_upgrade hook determines anything your canister
/// should do before it goes offline for a code upgrade.
fn pre_upgrade() {
    // ...
}

#[post_upgrade]
/// The post_upgrade hook determines anything your canister should do after it restarts
fn post_upgrade() {
    // ...
}

fn bls12_381_test_key_1() -> VetKDKeyId {
    VetKDKeyId {
        curve: VetKDCurve::Bls12_381,
        name: "test_key_1".to_string(),
    }
}

fn vetkd_system_api_canister_id() -> CanisterId {
    CanisterId::from_str(VETKD_SYSTEM_API_CANISTER_ID).expect("failed to create canister ID")
}

fn debug_println_caller(method_name: &str) {
    ic_cdk::println!(
        "{}: caller: {} (isAnonymous: {})",
        method_name,
        ic_cdk::caller().to_text(),
        ic_cdk::caller() == Principal::anonymous()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_tx() {
        todo!();
    }
}

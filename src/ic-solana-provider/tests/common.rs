#![allow(dead_code)]
use {
    candid::{decode_args, encode_one, utils::ArgumentDecoder, Principal},
    ic_solana_provider::state::InitArgs,
    lazy_static::lazy_static,
    pocket_ic::{nonblocking::PocketIc, CanisterSettings, WasmResult},
    rand::distributions::{Distribution, Standard},
    std::sync::Arc,
};

lazy_static! {
    pub static ref CONTROLLER_PRINCIPAL: Principal = random_principal();
    pub static ref USER_PRINCIPAL: Principal = random_principal();
    /// A basic identity for the Agent interactions with the canister that requires not anonymous calls.
    pub static ref BASIC_IDENTITY: Arc<dyn ic_agent::identity::Identity> =
        Arc::new(ic_agent::identity::BasicIdentity::from_key_pair(
            ring::signature::Ed25519KeyPair::from_pkcs8(&[
                48, 81, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 61, 228, 116, 88, 42,
                124, 214, 226, 172, 194, 68, 131, 157, 207, 140, 223, 130, 68, 183, 244, 217, 47,
                123, 95, 181, 186, 226, 146, 104, 164, 124, 217, 129, 33, 0, 193, 126, 113, 228,
                225, 79, 135, 250, 182, 90, 251, 237, 192, 11, 204, 52, 171, 23, 4, 0, 65, 12, 50,
                217, 158, 150, 124, 191, 138, 167, 12, 197,
            ])
            .unwrap()
        ));

    /// The static canister ID for the IC Solana Provider.
    pub static ref CANISTER_ID: Principal = Principal::from_slice(&[255, 255, 255, 255, 255, 208, 0, 1, 1, 1]);
}

#[ctor::ctor]
fn init_vars() {
    std::env::set_var("SCHNORR_CANISTER_PATH", "schnorr_canister.wasm.gz");
    std::env::set_var("IC_SOLANA_PROVIDER_PATH", "ic-solana-provider.wasm.gz");
}

// 2T cycles
const INIT_CYCLES: u128 = 2_000_000_000_000;

// The following secrets and their corresponding pubkeys have some funds in the devnet cluster.
pub const SECRET1: [u8; 64] = [
    201, 151, 232, 219, 29, 78, 185, 179, 104, 50, 186, 43, 6, 42, 140, 196, 114, 22, 49, 40, 233,
    30, 7, 12, 78, 27, 185, 87, 208, 213, 143, 157, 131, 153, 99, 40, 121, 85, 91, 41, 95, 59, 33,
    44, 51, 213, 148, 229, 38, 18, 43, 86, 174, 88, 90, 142, 20, 111, 151, 247, 215, 211, 234, 94,
];

pub const SECRET2: [u8; 64] = [
    251, 176, 190, 6, 118, 110, 35, 115, 34, 199, 117, 26, 113, 184, 159, 70, 99, 208, 216, 190,
    165, 159, 26, 221, 183, 167, 81, 153, 208, 152, 17, 108, 201, 195, 126, 216, 48, 140, 206, 211,
    127, 237, 43, 153, 6, 55, 239, 6, 147, 185, 60, 71, 45, 31, 170, 109, 42, 97, 217, 193, 21,
    234, 131, 118,
];

pub const PUBKEY1: &str = "9ri4mUToddwCc6jg1GTL5sobkkFxjUzjZ6CZ6L91LzAR";
pub const PUBKEY2: &str = "EabqyjABpFwUGhw2t2HVPGavjD1uqGm6ciMPhBRrdTxh";

pub const SOLANA_MAINNET_CLUSTER_URL: &str = "https://api.mainnet-beta.solana.com";
pub const SOLANA_DEVNET_CLUSTER_URL: &str = "https://api.devnet.solana.com";
pub const SOLANA_TESTNET_CLUSTER_URL: &str = "https://api.testnet.solana.com";

pub async fn init(ic: &PocketIc) -> Principal {
    init_with_rpc_url(ic, SOLANA_MAINNET_CLUSTER_URL).await
}

pub async fn init_with_rpc_url(ic: &PocketIc, rpc_url: &str) -> Principal {
    let (schnorr_canister_id, wasm_module) = create_canister(ic, "SCHNORR_CANISTER_PATH").await;

    ic.install_canister(
        schnorr_canister_id,
        wasm_module,
        vec![],
        Some(CONTROLLER_PRINCIPAL.clone()),
    )
    .await;
    fast_forward(ic, 5).await;

    let (canister_id, wasm_module) =
        create_canister_with_id(ic, "IC_SOLANA_PROVIDER_PATH", CANISTER_ID.clone()).await;

    let args = InitArgs {
        rpc_url: Some(rpc_url.to_string()),
        nodes_in_subnet: None,
        schnorr_canister: Some(schnorr_canister_id.to_string()),
        schnorr_key_name: None,
    };

    ic.install_canister(
        canister_id,
        wasm_module,
        encode_one(args).unwrap(),
        Some(CONTROLLER_PRINCIPAL.clone()),
    )
    .await;
    fast_forward(ic, 5).await;

    canister_id
}

pub async fn create_canister(ic: &PocketIc, env_key: &str) -> (Principal, Vec<u8>) {
    let canister_id = ic
        .create_canister_with_settings(
            Some(CONTROLLER_PRINCIPAL.clone()),
            Some(CanisterSettings {
                controllers: Some(vec![CONTROLLER_PRINCIPAL.clone()]),
                ..CanisterSettings::default()
            }),
        )
        .await;

    ic.add_cycles(canister_id, INIT_CYCLES).await;

    let wasm_path =
        std::env::var_os(env_key).unwrap_or_else(|| panic!("Missing `{}` env variable", env_key));

    let wasm_module = std::fs::read(wasm_path).unwrap();
    (canister_id, wasm_module)
}

pub async fn create_canister_with_id(
    ic: &PocketIc,
    env_key: &str,
    canister_id: Principal,
) -> (Principal, Vec<u8>) {
    ic.create_canister_with_id(
        Some(CONTROLLER_PRINCIPAL.clone()),
        Some(CanisterSettings {
            controllers: Some(vec![CONTROLLER_PRINCIPAL.clone()]),
            ..CanisterSettings::default()
        }),
        canister_id,
    )
    .await
    .unwrap();

    ic.add_cycles(canister_id, INIT_CYCLES).await;

    let wasm_path =
        std::env::var_os(env_key).unwrap_or_else(|| panic!("Missing `{}` env variable", env_key));

    let wasm_module = std::fs::read(wasm_path).unwrap();
    (canister_id, wasm_module)
}

pub fn random<T>() -> T
where
    Standard: Distribution<T>,
{
    rand::random()
}

pub fn random_principal() -> Principal {
    let random_bytes = random::<u32>().to_ne_bytes();

    Principal::from_slice(&random_bytes)
}

pub async fn fast_forward(ic: &PocketIc, ticks: u64) {
    for _ in 0..ticks - 1 {
        ic.tick().await;
    }
}

pub fn decode_raw_wasm_result<'a, Tuple>(data: &'a WasmResult) -> candid::Result<Tuple>
where
    Tuple: ArgumentDecoder<'a>,
{
    match data {
        WasmResult::Reply(data) => decode_args::<'a, Tuple>(&data),
        WasmResult::Reject(error_message) => Err(candid::Error::Custom(anyhow::anyhow!(
            error_message.clone()
        ))),
    }
}

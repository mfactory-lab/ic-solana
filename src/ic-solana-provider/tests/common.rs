use {
    candid::{decode_args, decode_one, encode_one, utils::ArgumentDecoder, CandidType, Principal},
    ic_solana_provider::state::InitArgs,
    lazy_static::lazy_static,
    pocket_ic::{CanisterSettings, PocketIc, WasmResult},
    rand::distributions::{Distribution, Standard},
    serde::Deserialize,
};

lazy_static! {
    pub static ref CONTROLLER_PRINCIPAL: Principal = random_principal();
    pub static ref USER_PRINCIPAL: Principal = random_principal();
}

// 2T cycles
const INIT_CYCLES: u128 = 2_000_000_000_000;

pub fn init(ic: &PocketIc) -> Principal {
    let (schnorr_canister_id, wasm_module) = create_canister(ic, "SCHNORR_CANISTER_PATH");

    ic.install_canister(
        schnorr_canister_id,
        wasm_module,
        vec![],
        Some(CONTROLLER_PRINCIPAL.clone()),
    );
    fast_forward(ic, 5);

    let (canister_id, wasm_module) = create_canister(ic, "IC_SOLANA_PROVIDER_PATH");

    let args = InitArgs {
        rpc_url: None,
        nodes_in_subnet: None,
        schnorr_canister: Some(schnorr_canister_id.to_string()),
        schnorr_key_name: None,
    };

    ic.install_canister(
        canister_id,
        wasm_module,
        encode_one(args).unwrap(),
        Some(CONTROLLER_PRINCIPAL.clone()),
    );
    fast_forward(ic, 5);

    canister_id
}

pub fn create_canister(ic: &PocketIc, env_key: &str) -> (Principal, Vec<u8>) {
    let canister_id = ic.create_canister_with_settings(
        Some(CONTROLLER_PRINCIPAL.clone()),
        Some(CanisterSettings {
            controllers: Some(vec![CONTROLLER_PRINCIPAL.clone()]),
            ..CanisterSettings::default()
        }),
    );

    ic.add_cycles(canister_id, INIT_CYCLES);

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

pub fn update<T: CandidType + for<'de> Deserialize<'de>>(
    ic: &PocketIc,
    sender: Principal,
    canister: Principal,
    method: &str,
    args: Vec<u8>,
) -> Result<T, String> {
    match ic.update_call(canister, sender, method, args) {
        Ok(WasmResult::Reply(data)) => decode_one(&data).unwrap(),
        Ok(WasmResult::Reject(error_message)) => Err(error_message.to_string()),
        Err(user_error) => Err(user_error.to_string()),
    }
}

pub fn query<T: CandidType + for<'de> Deserialize<'de>>(
    ic: &PocketIc,
    sender: Principal,
    canister: Principal,
    method: &str,
    args: Vec<u8>,
) -> Result<T, String> {
    match ic.query_call(canister, sender, method, args) {
        Ok(WasmResult::Reply(data)) => decode_one(&data).unwrap(),
        Ok(WasmResult::Reject(error_message)) => Err(error_message.to_string()),
        Err(user_error) => Err(user_error.to_string()),
    }
}

pub fn fast_forward(ic: &PocketIc, ticks: u64) {
    for _ in 0..ticks - 1 {
        ic.tick();
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

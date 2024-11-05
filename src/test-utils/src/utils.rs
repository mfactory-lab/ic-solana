use ic_test_utilities_load_wasm::load_wasm;
use pocket_ic::WasmResult;

pub fn load_wasm_by_name(name: &str) -> Vec<u8> {
    load_wasm(std::env::var("CARGO_MANIFEST_DIR").unwrap(), name, &[])
}

pub fn load_wasm_by_env_var(env_var: &str) -> Vec<u8> {
    let wasm_path = std::env::var(env_var)
        .unwrap_or_else(|e| panic!("The wasm path must be set using the env variable {} ({})", env_var, e));
    std::fs::read(&wasm_path).unwrap_or_else(|e| {
        panic!(
            "Failed to load Wasm file from path {} (env var {}): {}",
            wasm_path, env_var, e
        )
    })
}

/// Asserts that the result is a successful reply.
pub fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        result => {
            panic!("Expected a successful reply, got {:?}", result)
        }
    }
}

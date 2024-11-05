#![allow(dead_code)]

use std::path::PathBuf;

use candid::{utils::ArgumentEncoder, CandidType, Principal};
use ic_solana_wallet::state::InitArgs;
use ic_test_utilities_load_wasm::load_wasm;
use serde::de::DeserializeOwned;
use test_utils::{CallFlow, TestSetup};

thread_local! {
     static RPC_WASM: Vec<u8> = load_wasm(get_root(), "ic-solana-rpc", &[]);
     static WASM: Vec<u8> = load_wasm(env!("CARGO_MANIFEST_DIR"), env!("CARGO_PKG_NAME"), &[]);
}

#[derive(Clone)]
pub struct SolanaWalletSetup {
    rpc_setup: TestSetup,
    setup: TestSetup,
}

impl Default for SolanaWalletSetup {
    fn default() -> Self {
        Self::new()
    }
}

impl SolanaWalletSetup {
    pub fn new() -> Self {
        let rpc_setup = TestSetup::new(
            RPC_WASM.with(|wasm| wasm.clone()),
            ic_solana_rpc::state::InitArgs {
                demo: Some(true),
                managers: Some(vec![TestSetup::controller_id()]),
            },
        );

        let sol_canister = Some(rpc_setup.canister_id);

        Self {
            rpc_setup,
            setup: TestSetup::new(
                WASM.with(|wasm| wasm.clone()),
                InitArgs {
                    sol_canister,
                    schnorr_key: None,
                },
            ),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn as_controller(mut self) -> Self {
        self.setup = self.setup.as_controller();
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn as_caller<T: Into<Principal>>(mut self, id: T) -> Self {
        self.setup = self.setup.as_caller(id);
        self
    }

    pub fn upgrade_canister(&self, args: InitArgs) {
        self.setup.upgrade_canister(WASM.with(|wasm| wasm.clone()), args)
    }

    pub fn call_update<A: ArgumentEncoder, R: CandidType + DeserializeOwned>(
        &self,
        method: &str,
        args: A,
    ) -> CallFlow<R> {
        self.setup.call_update(method, args)
    }

    pub fn call_query<A: ArgumentEncoder, R: CandidType + DeserializeOwned>(&self, method: &str, args: A) -> R {
        self.setup.call_query(method, args)
    }
}

/// Retrieves the project root directory.
fn get_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("Invalid project root path")
}

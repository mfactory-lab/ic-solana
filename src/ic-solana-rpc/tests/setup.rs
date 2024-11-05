use candid::{utils::ArgumentEncoder, CandidType, Decode, Encode, Principal};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_solana::{
    logs::{Log, LogEntry},
    metrics::Metrics,
    rpc_client::{RpcResult, RpcServices},
};
use ic_solana_rpc::{
    auth::Auth,
    state::InitArgs,
    types::{RegisterProviderArgs, UpdateProviderArgs},
};
use ic_test_utilities_load_wasm::load_wasm;
use serde::de::DeserializeOwned;
use test_utils::{utils::assert_reply, CallFlow, TestSetup};

pub const MOCK_RAW_TX: &str ="4hXTCkRzt9WyecNzV1XPgCDfGAZzQKNxLXgynz5QDuWWPSAZBZSHptvWRL3BjCvzUXRdKvHL2b7yGrRQcWyaqsaBCncVG7BFggS8w9snUts67BSh3EqKpXLUm5UMHfD7ZBe9GhARjbNQMLJ1QD3Spr6oMTBU6EhdB4RD8CP2xUxr2u3d6fos36PD98XS6oX8TQjLpsMwncs5DAMiD4nNnR8NBfyghGCWvCVifVwvA8B8TJxE1aiyiv2L429BCWfyzAme5sZW8rDb14NeCQHhZbtNqfXhcp2tAnaAT";

thread_local! {
     static WASM: Vec<u8> = load_wasm(env!("CARGO_MANIFEST_DIR"), env!("CARGO_PKG_NAME"), &[]);
}

/// Creates a mock update call.
pub fn mock_update<A: ArgumentEncoder, R: DeserializeOwned + CandidType>(
    method: &str,
    args: A,
    response: &str,
) -> RpcResult<R> {
    SolanaRpcSetup::default()
        .call_update::<A, RpcResult<R>>(method, args)
        .mock_http(test_utils::MockOutcallBuilder::new(200, response))
        .wait()
}

#[derive(Clone)]
pub struct SolanaRpcSetup {
    setup: TestSetup,
}

impl Default for SolanaRpcSetup {
    fn default() -> Self {
        Self::new(InitArgs {
            demo: Some(true),
            managers: Some(vec![TestSetup::controller_id()]),
        })
    }
}

impl SolanaRpcSetup {
    pub fn new(args: InitArgs) -> Self {
        Self {
            setup: TestSetup::new(WASM.with(|wasm| wasm.clone()), args),
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

    pub fn get_metrics(&self) -> Metrics {
        self.call_query("getMetrics", ())
    }

    pub fn http_get_logs(&self, priority: &str) -> Vec<LogEntry> {
        let request = HttpRequest {
            method: "".to_string(),
            url: format!("/logs?priority={priority}"),
            headers: vec![],
            body: serde_bytes::ByteBuf::new(),
        };
        let response = Decode!(
            &assert_reply(
                self.setup
                    .env
                    .query_call(
                        self.setup.canister_id,
                        Principal::anonymous(),
                        "http_request",
                        Encode!(&request).unwrap()
                    )
                    .expect("failed to get logs")
            ),
            HttpResponse
        )
        .unwrap();
        serde_json::from_slice::<Log>(&response.body)
            .expect("failed to parse logs response")
            .entries
    }

    pub fn get_nodes_in_subnet(&self) -> u32 {
        self.call_query("getNodesInSubnet", ())
    }

    pub fn request(
        &self,
        source: RpcServices,
        method: &str,
        params: &str,
        max_response_bytes: u64,
    ) -> CallFlow<RpcResult<String>> {
        self.setup
            .call_update("request", (source, method, params, max_response_bytes))
    }

    pub fn get_providers(&self) -> Vec<String> {
        self.setup.call_query("getProviders", ())
    }

    pub fn register_provider(&self, args: RegisterProviderArgs) -> CallFlow<()> {
        self.setup.call_update("registerProvider", (args,))
    }

    pub fn unregister_provider(&self, id: &str) -> CallFlow<bool> {
        self.setup.call_update("unregisterProvider", (id,))
    }

    #[allow(dead_code)]
    pub fn update_provider(&self, args: UpdateProviderArgs) -> CallFlow<()> {
        self.setup.call_update("updateProvider", (args,))
    }

    pub fn get_authorized(&self, auth: Auth) -> Vec<Principal> {
        self.setup.call_query("getAuthorized", (auth,))
    }

    pub fn authorize(&self, principal: Principal, auth: Auth) -> CallFlow<bool> {
        self.setup.call_update("authorize", (principal, auth))
    }

    pub fn deauthorize(&self, principal: Principal, auth: Auth) -> CallFlow<bool> {
        self.setup.call_update("deauthorize", (principal, auth))
    }
}

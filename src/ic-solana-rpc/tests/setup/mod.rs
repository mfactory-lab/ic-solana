#![allow(dead_code)]

mod mock;

pub use mock::*;
use {
    candid::{encode_args, utils::ArgumentEncoder, CandidType, Decode, Encode, Principal},
    ic_canisters_http_types::{HttpRequest, HttpResponse},
    ic_cdk::api::management_canister::main::CanisterId,
    ic_solana::{
        logs::{Log, LogEntry},
        metrics::Metrics,
        rpc_client::{RpcResult, RpcServices},
    },
    ic_solana_rpc::{
        auth::Auth,
        state::InitArgs,
        types::{RegisterProviderArgs, UpdateProviderArgs},
    },
    ic_test_utilities_load_wasm::load_wasm,
    pocket_ic::{
        common::rest::{CanisterHttpResponse, MockCanisterHttpResponse, RawMessageId},
        CanisterSettings, PocketIc, WasmResult,
    },
    rand::distributions::{Distribution, Standard},
    serde::de::DeserializeOwned,
    std::{marker::PhantomData, sync::Arc, time::Duration},
};

const DEFAULT_CALLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x01]);
const DEFAULT_CONTROLLER_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
pub const ADDITIONAL_TEST_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x03]);
const INITIAL_CYCLES: u128 = 100_000_000_000_000_000;
const MAX_TICKS: usize = 10;

pub const MOCK_RAW_TX: &str ="4hXTCkRzt9WyecNzV1XPgCDfGAZzQKNxLXgynz5QDuWWPSAZBZSHptvWRL3BjCvzUXRdKvHL2b7yGrRQcWyaqsaBCncVG7BFggS8w9snUts67BSh3EqKpXLUm5UMHfD7ZBe9GhARjbNQMLJ1QD3Spr6oMTBU6EhdB4RD8CP2xUxr2u3d6fos36PD98XS6oX8TQjLpsMwncs5DAMiD4nNnR8NBfyghGCWvCVifVwvA8B8TJxE1aiyiv2L429BCWfyzAme5sZW8rDb14NeCQHhZbtNqfXhcp2tAnaAT";

// The following secrets and their corresponding pubkeys have some funds in the devnet cluster.
pub const SECRET1: [u8; 64] = [
    201, 151, 232, 219, 29, 78, 185, 179, 104, 50, 186, 43, 6, 42, 140, 196, 114, 22, 49, 40, 233, 30, 7, 12, 78, 27,
    185, 87, 208, 213, 143, 157, 131, 153, 99, 40, 121, 85, 91, 41, 95, 59, 33, 44, 51, 213, 148, 229, 38, 18, 43, 86,
    174, 88, 90, 142, 20, 111, 151, 247, 215, 211, 234, 94,
];

pub const SECRET2: [u8; 64] = [
    251, 176, 190, 6, 118, 110, 35, 115, 34, 199, 117, 26, 113, 184, 159, 70, 99, 208, 216, 190, 165, 159, 26, 221,
    183, 167, 81, 153, 208, 152, 17, 108, 201, 195, 126, 216, 48, 140, 206, 211, 127, 237, 43, 153, 6, 55, 239, 6, 147,
    185, 60, 71, 45, 31, 170, 109, 42, 97, 217, 193, 21, 234, 131, 118,
];

pub const PUBKEY1: &str = "9ri4mUToddwCc6jg1GTL5sobkkFxjUzjZ6CZ6L91LzAR";
pub const PUBKEY2: &str = "EabqyjABpFwUGhw2t2HVPGavjD1uqGm6ciMPhBRrdTxh";

pub fn load_wasm_by_name(name: &str) -> Vec<u8> {
    load_wasm(std::env::var("CARGO_MANIFEST_DIR").unwrap(), name, &[])
}

pub fn load_wasm_using_env_var(env_var: &str) -> Vec<u8> {
    let wasm_path = std::env::var(env_var)
        .unwrap_or_else(|e| panic!("The wasm path must be set using the env variable {} ({})", env_var, e));
    std::fs::read(&wasm_path).unwrap_or_else(|e| {
        panic!(
            "failed to load Wasm file from path {} (env var {}): {}",
            wasm_path, env_var, e
        )
    })
}

fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        result => {
            panic!("Expected a successful reply, got {:?}", result)
        }
    }
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

#[derive(Clone)]
pub struct SolanaRpcSetup {
    pub env: Arc<PocketIc>,
    pub caller: Principal,
    pub controller: Principal,
    pub canister_id: CanisterId,
}

impl Default for SolanaRpcSetup {
    fn default() -> Self {
        Self::new()
    }
}

impl SolanaRpcSetup {
    pub fn new() -> Self {
        Self::new_with_args(InitArgs {
            demo: Some(true),
            managers: Some(vec![DEFAULT_CONTROLLER_TEST_ID]),
        })
    }

    pub fn new_with_args(args: InitArgs) -> Self {
        let env = Arc::new(PocketIc::new());

        let caller = DEFAULT_CALLER_TEST_ID;
        let controller = DEFAULT_CONTROLLER_TEST_ID;
        let canister_id = env.create_canister_with_settings(
            None,
            Some(CanisterSettings {
                controllers: Some(vec![controller]),
                ..CanisterSettings::default()
            }),
        );
        env.add_cycles(canister_id, INITIAL_CYCLES);
        env.install_canister(
            canister_id,
            Self::wasm_bytes(),
            Encode!(&args).unwrap(),
            Some(controller),
        );

        Self {
            env,
            caller,
            controller,
            canister_id,
        }
    }

    pub fn as_controller(mut self) -> Self {
        self.caller = self.controller;
        self
    }

    pub fn as_caller<T: Into<Principal>>(mut self, id: T) -> Self {
        self.caller = id.into();
        self
    }

    fn wasm_bytes() -> Vec<u8> {
        load_wasm_by_name("ic-solana-rpc")
    }

    pub fn upgrade_canister(&self, args: InitArgs) {
        self.env.tick();
        // Avoid `CanisterInstallCodeRateLimited` error
        self.env.advance_time(Duration::from_secs(600));
        self.env.tick();
        self.env
            .upgrade_canister(
                self.canister_id,
                Self::wasm_bytes(),
                Encode!(&args).unwrap(),
                Some(self.controller),
            )
            .expect("Error while upgrading canister");
    }

    pub fn call_update<A: ArgumentEncoder, R: CandidType + DeserializeOwned>(
        &self,
        method: &str,
        args: A,
    ) -> CallFlow<R> {
        let input = encode_args(args).unwrap();
        CallFlow::from_update(self.clone(), method, input)
    }

    pub fn call_query<A: ArgumentEncoder, R: CandidType + DeserializeOwned>(&self, method: &str, args: A) -> R {
        let input = encode_args(args).unwrap();
        let candid = &assert_reply(
            self.env
                .query_call(self.canister_id, self.caller, method, input)
                .unwrap_or_else(|err| panic!("error during query call to `{}()`: {}", method, err)),
        );
        Decode!(candid, R).expect("error while decoding Candid response from query call")
    }

    pub fn tick_until_http_request(&self) {
        for _ in 0..MAX_TICKS {
            if !self.env.get_canister_http().is_empty() {
                break;
            }
            self.env.tick();
            self.env.advance_time(Duration::from_nanos(1));
        }
    }

    pub fn get_metrics(&self) -> Metrics {
        self.call_query("getMetrics", ())
    }

    // pub fn mock_api_keys(self) -> Self {
    //     self.clone().as_controller().update_api_keys(
    //         &PROVIDERS
    //             .iter()
    //             .filter_map(|provider| {
    //                 Some((
    //                     provider.provider_id,
    //                     match provider.access {
    //                         RpcAccess::Authenticated { .. } => Some(MOCK_API_KEY.to_string()),
    //                         RpcAccess::Unauthenticated { .. } => None?,
    //                     },
    //                 ))
    //             })
    //             .collect::<Vec<_>>(),
    //     );
    //     self
    // }

    pub fn http_get_logs(&self, priority: &str) -> Vec<LogEntry> {
        let request = HttpRequest {
            method: "".to_string(),
            url: format!("/logs?priority={priority}"),
            headers: vec![],
            body: serde_bytes::ByteBuf::new(),
        };
        let response = Decode!(
            &assert_reply(
                self.env
                    .query_call(
                        self.canister_id,
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

    pub fn request(
        &self,
        source: RpcServices,
        method: &str,
        params: &str,
        max_response_bytes: u64,
    ) -> CallFlow<RpcResult<String>> {
        self.call_update("request", (source, method, params, max_response_bytes))
    }

    pub fn get_providers(&self) -> Vec<String> {
        self.call_query("getProviders", ())
    }

    pub fn register_provider(&self, args: RegisterProviderArgs) -> CallFlow<()> {
        self.call_update("registerProvider", (args,))
    }

    pub fn unregister_provider(&self, id: &str) -> CallFlow<bool> {
        self.call_update("unregisterProvider", (id,))
    }

    pub fn update_provider(&self, args: UpdateProviderArgs) -> CallFlow<()> {
        self.call_update("updateProvider", (args,))
    }

    pub fn get_authorized(&self, auth: Auth) -> Vec<Principal> {
        self.call_query("getAuthorized", (auth,))
    }

    pub fn authorize(&self, principal: Principal, auth: Auth) -> CallFlow<bool> {
        self.call_update("authorize", (principal, auth))
    }

    pub fn deauthorize(&self, principal: Principal, auth: Auth) -> CallFlow<bool> {
        self.call_update("deauthorize", (principal, auth))
    }
}

pub struct CallFlow<R> {
    setup: SolanaRpcSetup,
    method: String,
    message_id: RawMessageId,
    phantom: PhantomData<R>,
}

impl<R: CandidType + DeserializeOwned> CallFlow<R> {
    pub fn from_update(setup: SolanaRpcSetup, method: &str, input: Vec<u8>) -> Self {
        let message_id = setup
            .env
            .submit_call(setup.canister_id, setup.caller, method, input.clone())
            .expect("failed to submit call");
        Self::new(setup, method, message_id)
    }

    pub fn new(setup: SolanaRpcSetup, method: impl ToString, message_id: RawMessageId) -> Self {
        Self {
            setup,
            method: method.to_string(),
            message_id,
            phantom: Default::default(),
        }
    }

    pub fn mock_http(self, mock: impl Into<MockOutcall>) -> Self {
        let mock = mock.into();
        self.mock_http_once_inner(&mock);
        loop {
            if !self.try_mock_http_inner(&mock) {
                break;
            }
        }
        self
    }

    pub fn mock_http_n_times(self, mock: impl Into<MockOutcall>, count: u32) -> Self {
        let mock = mock.into();
        for _ in 0..count {
            self.mock_http_once_inner(&mock);
        }
        self
    }

    pub fn mock_http_once(self, mock: impl Into<MockOutcall>) -> Self {
        let mock = mock.into();
        self.mock_http_once_inner(&mock);
        self
    }

    fn mock_http_once_inner(&self, mock: &MockOutcall) {
        if !self.try_mock_http_inner(mock) {
            panic!("no pending HTTP request")
        }
    }

    fn try_mock_http_inner(&self, mock: &MockOutcall) -> bool {
        if self.setup.env.get_canister_http().is_empty() {
            self.setup.tick_until_http_request();
        }
        let http_requests = self.setup.env.get_canister_http();

        let request = match http_requests.first() {
            Some(request) => request,
            None => return false,
        };
        mock.assert_matches(request);

        let response = MockCanisterHttpResponse {
            subnet_id: request.subnet_id,
            request_id: request.request_id,
            response: CanisterHttpResponse::CanisterHttpReply(mock.response.clone()),
            additional_responses: vec![],
        };
        self.setup.env.mock_canister_http_response(response);
        true
    }

    pub fn wait(self) -> R {
        let candid = &assert_reply(
            self.setup
                .env
                .await_call(self.message_id)
                .unwrap_or_else(|err| panic!("error during update call to `{}()`: {}", self.method, err)),
        );
        Decode!(candid, R).expect("error while decoding Candid response from update call")
    }
}

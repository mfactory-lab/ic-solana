#![allow(dead_code)]

mod mock;
mod utils;

use std::{marker::PhantomData, sync::Arc, time::Duration};

use candid::{encode_args, utils::ArgumentEncoder, CandidType, Decode, Encode, Principal};
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_cdk::api::management_canister::main::CanisterId;
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
pub use mock::*;
use pocket_ic::{
    common::rest::{CanisterHttpResponse, MockCanisterHttpResponse, RawMessageId},
    CanisterSettings, PocketIc,
};
use serde::de::DeserializeOwned;

use crate::setup::utils::{assert_reply, load_wasm_by_name};

const CONTROLLER_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x02]);
const CALLER_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x01]);
pub const SOME_CALLER_ID: Principal = Principal::from_slice(&[0x9d, 0xf7, 0x03]);

const INITIAL_CYCLES: u128 = 100_000_000_000_000_000;
const MAX_TICKS: usize = 10;

pub const MOCK_RAW_TX: &str ="4hXTCkRzt9WyecNzV1XPgCDfGAZzQKNxLXgynz5QDuWWPSAZBZSHptvWRL3BjCvzUXRdKvHL2b7yGrRQcWyaqsaBCncVG7BFggS8w9snUts67BSh3EqKpXLUm5UMHfD7ZBe9GhARjbNQMLJ1QD3Spr6oMTBU6EhdB4RD8CP2xUxr2u3d6fos36PD98XS6oX8TQjLpsMwncs5DAMiD4nNnR8NBfyghGCWvCVifVwvA8B8TJxE1aiyiv2L429BCWfyzAme5sZW8rDb14NeCQHhZbtNqfXhcp2tAnaAT";

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
            managers: Some(vec![CONTROLLER_ID]),
        })
    }

    pub fn new_with_args(args: InitArgs) -> Self {
        let env = Arc::new(PocketIc::new());

        let caller = CALLER_ID;
        let controller = CONTROLLER_ID;
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

    pub fn call_query<A: ArgumentEncoder, R: CandidType + DeserializeOwned>(
        &self,
        method: &str,
        args: A,
    ) -> R {
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
        let candid = &assert_reply(self.setup.env.await_call(self.message_id).unwrap_or_else(
            |err| panic!("error during update call to `{}()`: {}", self.method, err),
        ));
        Decode!(candid, R).expect("error while decoding Candid response from update call")
    }
}

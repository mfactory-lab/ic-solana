#![allow(dead_code)]

pub mod mock;
pub mod utils;

use std::{marker::PhantomData, sync::Arc, time::Duration};

use candid::{encode_args, utils::ArgumentEncoder, CandidType, Decode, Encode, Principal};
use ic_cdk::api::management_canister::main::CanisterId;
pub use mock::*;
use pocket_ic::{
    common::rest::{CanisterHttpResponse, MockCanisterHttpResponse, RawMessageId},
    management_canister::CanisterSettings,
    PocketIc, PocketIcBuilder,
};
use serde::de::DeserializeOwned;
use utils::assert_reply;

const INITIAL_CYCLES: u128 = 100_000_000_000_000_000;
const UPGRADE_TIMEOUT: u64 = 600;
const MAX_TICKS: usize = 10;

#[derive(Clone)]
pub struct TestSetup {
    pub env: Arc<PocketIc>,
    pub caller: Principal,
    pub controller: Principal,
    pub canister_id: CanisterId,
}

impl TestSetup {
    pub fn new<T: CandidType>(wasm: Vec<u8>, args: T) -> Self {
        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_ii_subnet()
            .with_application_subnet()
            .build();

        let env = Arc::new(pic);
        let caller = Self::caller_id();
        let controller = Self::controller_id();
        let canister_id = env.create_canister_with_settings(
            None,
            Some(CanisterSettings {
                controllers: Some(vec![controller]),
                ..CanisterSettings::default()
            }),
        );
        env.add_cycles(canister_id, INITIAL_CYCLES);
        env.install_canister(canister_id, wasm, Encode!(&args).unwrap(), Some(controller));

        Self {
            env,
            caller,
            controller,
            canister_id,
        }
    }

    pub const fn caller_id() -> Principal {
        Self::principal(0x01)
    }

    pub const fn controller_id() -> Principal {
        Self::principal(0x02)
    }

    /// Creates a principal with the given id
    pub const fn principal(id: u8) -> Principal {
        Principal::from_slice(&[0x9d, 0xf7, id])
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn as_controller(mut self) -> Self {
        self.caller = self.controller;
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn as_caller<T: Into<Principal>>(mut self, id: T) -> Self {
        self.caller = id.into();
        self
    }

    pub fn upgrade_canister<A: CandidType>(&self, wasm: Vec<u8>, args: A) {
        self.env.tick();
        // Avoid `CanisterInstallCodeRateLimited` error
        self.env.advance_time(Duration::from_secs(UPGRADE_TIMEOUT));
        self.env.tick();
        self.env
            .upgrade_canister(self.canister_id, wasm, Encode!(&args).unwrap(), Some(self.controller))
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
}

pub struct CallFlow<R> {
    setup: TestSetup,
    method: String,
    message_id: RawMessageId,
    phantom: PhantomData<R>,
}

impl<R: CandidType + DeserializeOwned> CallFlow<R> {
    pub fn from_update(setup: TestSetup, method: &str, input: Vec<u8>) -> Self {
        let message_id = setup
            .env
            .submit_call(setup.canister_id, setup.caller, method, input.clone())
            .expect("failed to submit call");
        Self::new(setup, method, message_id)
    }

    pub fn new(setup: TestSetup, method: impl ToString, message_id: RawMessageId) -> Self {
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

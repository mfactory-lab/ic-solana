use std::cell::RefCell;
use std::env;

thread_local! {
    pub static STATE: RefCell<Option<State>> = RefCell::default();
}

use crate::cluster::Cluster;
use crate::constants::NODES_IN_FIDUCIARY_SUBNET;
use candid::{CandidType, Deserialize};

// https://github.com/domwoe/schnorr_canister/blob/502a263c01902a1154ef354aefa161795a669de1/src/lib.rs#L54
const DEFAULT_KEY_NAME: &str = "test_key_1";

/// Solana RPC canister initialization data.
#[derive(Debug, Deserialize, CandidType, Clone)]
pub struct InitArgs {
    pub rpc_url: Option<String>,
    pub nodes_in_subnet: Option<u32>,
    pub schnorr_canister: Option<String>,
    pub schnorr_key_name: Option<String>,
    pub key_name: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct State {
    pub rpc_url: String,
    pub schnorr_canister: String,
    pub schnorr_key_name: String,
    pub nodes_in_subnet: u32,
    pub http_request_counter: u64,
}

impl From<InitArgs> for State {
    fn from(value: InitArgs) -> Self {
        Self {
            rpc_url: value.rpc_url.unwrap_or(Cluster::Devnet.to_string()),
            schnorr_canister: env::var("CANISTER_ID_SCHNORR_CANISTER")
                .unwrap_or(value.schnorr_canister.expect("Missing schnorr_canister")),
            schnorr_key_name: value
                .schnorr_key_name
                .unwrap_or(DEFAULT_KEY_NAME.to_string()),
            nodes_in_subnet: value.nodes_in_subnet.unwrap_or(NODES_IN_FIDUCIARY_SUBNET),
            http_request_counter: 0,
        }
    }
}

impl State {
    pub fn next_request_id(&mut self) -> u64 {
        let current_request_id = self.http_request_counter;
        // overflow is not an issue here because we only use `next_request_id` to correlate
        // requests and responses in logs.
        self.http_request_counter = self.http_request_counter.wrapping_add(1);
        current_request_id
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Solana RPC URL: {:?}", self.rpc_url)?;
        writeln!(f, "Schnorr canister: {:?}", self.schnorr_canister)?;
        writeln!(f, "Schnorr key name: {:?}", self.schnorr_key_name)?;
        writeln!(f, "Nodes in subnet: {:?}", self.nodes_in_subnet)?;
        writeln!(f, "HTTP request counter: {:?}", self.http_request_counter)?;
        Ok(())
    }
}

/// Take the current state.
///
/// After calling this function, the state won't be initialized anymore.
/// Panics if there is no state.
pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(State) -> R,
{
    STATE.with(|s| f(s.take().expect("State not initialized!")))
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

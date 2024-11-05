use std::{cell::RefCell, str::FromStr};

use candid::{CandidType, Deserialize};
use ic_cdk::{
    api::management_canister::main::CanisterId,
    storage::{stable_restore, stable_save},
};
use serde::Serialize;

use crate::eddsa::SchnorrKey;

thread_local! {
    pub static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

#[derive(Debug, Deserialize, CandidType, Clone)]
pub struct InitArgs {
    pub sol_canister: Option<CanisterId>,
    pub schnorr_key: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct State {
    pub sol_canister: CanisterId,
    pub schnorr_key: SchnorrKey,
}

impl State {
    pub fn init(args: InitArgs) {
        replace_state(Self {
            sol_canister: args.sol_canister.expect("Missing sol_canister"),
            schnorr_key: args
                .schnorr_key
                .and_then(|s| SchnorrKey::from_str(&s).ok())
                .unwrap_or(SchnorrKey::TestKey1),
        });
    }

    pub fn pre_upgrade() {
        take_state(|state| stable_save((state,)).expect("failed to save state"))
    }

    pub fn post_upgrade(args: Option<InitArgs>) {
        let (mut state,): (State,) = stable_restore().expect("failed to restore state");
        if let Some(args) = args {
            if let Some(sol_canister) = args.sol_canister {
                state.sol_canister = sol_canister;
            }
            if let Some(schnorr_key) = args.schnorr_key {
                state.schnorr_key = SchnorrKey::from_str(&schnorr_key).expect("Invalid schnorr key");
            }
        }
        replace_state(state);
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Solana canister: {:?}", self.sol_canister)?;
        writeln!(f, "Schnorr key: {:?}", self.schnorr_key)?;
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

/// Replaces the current state.
pub fn replace_state(state: State) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

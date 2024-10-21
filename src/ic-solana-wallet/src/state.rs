use {
    crate::eddsa::SchnorrKey,
    candid::{CandidType, Deserialize, Principal},
    std::{cell::RefCell, str::FromStr},
};

thread_local! {
    pub static STATE: RefCell<Option<State>> = RefCell::new(None);
}

/// Solana RPC canister initialization data.
#[derive(Debug, Deserialize, CandidType, Clone)]
pub struct InitArgs {
    pub sol_canister: Principal,
    pub schnorr_key: Option<String>,
}

pub struct State {
    pub sol_canister: Principal,
    pub schnorr_key: SchnorrKey,
}

impl From<InitArgs> for State {
    fn from(value: InitArgs) -> Self {
        take_state(|s| Self {
            sol_canister: value.sol_canister,
            schnorr_key: value
                .schnorr_key
                .and_then(|s| SchnorrKey::from_str(&s).ok())
                .unwrap_or(s.schnorr_key),
        })
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

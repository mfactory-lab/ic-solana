use std::cell::RefCell;

thread_local! {
  pub static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Debug, PartialEq, Clone)]
pub struct State {
    pub rpc_url: String,
    pub nodes_in_subnet: u32,
    pub http_request_counter: u64,
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

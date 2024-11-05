use std::cell::RefCell;

use candid::{CandidType, Deserialize, Principal};
use ic_solana::types::Cluster;

use crate::{
    auth::{Auth, AuthSet},
    memory::{init_auth_memory, init_providers_memory, AuthMemory, ProvidersMemory},
    providers::{ProviderId, RpcProvider},
    types::PrincipalStorable,
};

thread_local! {
    pub static STATE: RefCell<Option<State>> = RefCell::new(Some(State {
        auth: init_auth_memory(),
        rpc_providers: init_providers_memory(),
        is_demo_active: false,
    }));
}

/// Solana RPC canister initialization data.
#[derive(Clone, Debug, Default, Deserialize, CandidType)]
pub struct InitArgs {
    pub demo: Option<bool>,
    pub managers: Option<Vec<Principal>>,
}

pub struct State {
    pub auth: AuthMemory,
    pub rpc_providers: ProvidersMemory,
    pub is_demo_active: bool,
    // pub hosts_blocklist: Vec<String>,
}

impl State {
    fn init_default_providers(providers: &mut ProvidersMemory) {
        for cluster in [Cluster::Mainnet, Cluster::Testnet, Cluster::Devnet] {
            providers.insert(
                ProviderId(cluster.to_string()),
                RpcProvider {
                    url: cluster.url().to_string(),
                    owner: ic_cdk::caller(),
                    auth: None,
                },
            );
        }
    }
    pub fn is_authorized(&self, principal: &Principal, auth: Auth) -> bool {
        if let Some(v) = self.auth.get(&PrincipalStorable(*principal)) {
            v.is_authorized(auth)
        } else {
            false
        }
    }
}

impl From<InitArgs> for State {
    fn from(value: InitArgs) -> Self {
        take_state(|s| {
            let mut auth = s.auth;
            if let Some(managers) = value.managers {
                for manager in managers {
                    auth.insert(PrincipalStorable(manager), AuthSet::new(vec![Auth::Manage]));
                }
            }

            let mut rpc_providers = s.rpc_providers;
            Self::init_default_providers(&mut rpc_providers);

            Self {
                auth,
                rpc_providers,
                is_demo_active: value.demo.unwrap_or(false),
                // hosts_blocklist: value.hosts_blocklist.unwrap_or_default(),
            }
        })
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Demo active: {:?}", self.is_demo_active)?;
        writeln!(f, "Auth:")?;
        for (principal, auth) in self.auth.iter() {
            writeln!(f, "  - {}: {:?}", principal.0, auth)?;
        }
        writeln!(f, "RPC providers:")?;
        for (provider_id, provider) in self.rpc_providers.iter() {
            writeln!(f, "  - {}: {:?}", provider_id.0, provider)?;
        }
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

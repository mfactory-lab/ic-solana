use std::{borrow::Cow, fmt::Display};

use candid::{CandidType, Deserialize, Principal};
use ic_canister_log::log;
use ic_solana::{add_metric_entry, logs::INFO, metrics::MetricAuth, sub_metric_entry};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;

use crate::{
    state::{mutate_state, read_state},
    types::PrincipalStorable,
};

#[derive(Clone, Copy, Debug, PartialEq, CandidType, Serialize, Deserialize)]
pub enum Auth {
    Manage,
    RegisterProvider,
    // FreeRpc,
}

impl Display for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Auth::Manage => "manage",
                Auth::RegisterProvider => "register_provider",
                // Auth::FreeRpc => "FreeRpc",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, CandidType, Serialize, Deserialize, Default)]
pub struct AuthSet(Vec<Auth>);

impl AuthSet {
    pub const MAX_SIZE: usize = 1000;

    pub fn new(auths: Vec<Auth>) -> Self {
        let mut auth_set = Self(Vec::with_capacity(auths.len()));
        for auth in auths {
            // Deduplicate
            auth_set.authorize(auth);
        }
        auth_set
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_authorized(&self, auth: Auth) -> bool {
        self.0.contains(&auth)
    }

    pub fn authorize(&mut self, auth: Auth) -> bool {
        if !self.is_authorized(auth) {
            self.0.push(auth);
            true
        } else {
            false
        }
    }

    pub fn deauthorize(&mut self, auth: Auth) -> bool {
        if let Some(index) = self.0.iter().position(|a| *a == auth) {
            self.0.remove(index);
            true
        } else {
            false
        }
    }
}

// Using explicit JSON representation in place of enum indices for security
impl Storable for AuthSet {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_json::to_vec(self).expect("Unable to serialize AuthSet"))
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_json::from_slice(&bytes).expect("Unable to deserialize AuthSet")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: Self::MAX_SIZE as u32,
        is_fixed_size: false,
    };
}

pub fn is_authorized(principal: &Principal, auth: Auth) -> bool {
    read_state(|s| {
        if let Some(v) = s.auth.get(&PrincipalStorable(*principal)) {
            v.is_authorized(auth)
        } else {
            false
        }
    })
}

pub fn require_manage_or_controller() -> Result<(), String> {
    let caller = ic_cdk::caller();
    if is_authorized(&caller, Auth::Manage) || ic_cdk::api::is_controller(&caller) {
        Ok(())
    } else {
        let auth = MetricAuth(Auth::Manage.to_string());
        add_metric_entry!(err_unauthorized, auth, 1);
        Err("Unauthorized".to_string())
    }
}

pub fn require_register_provider() -> Result<(), String> {
    if is_authorized(&ic_cdk::caller(), Auth::RegisterProvider) || require_manage_or_controller().is_ok() {
        Ok(())
    } else {
        let auth = MetricAuth(Auth::RegisterProvider.to_string());
        add_metric_entry!(err_unauthorized, auth, 1);
        Err("Unauthorized".to_string())
    }
}

pub fn do_authorize(principal: Principal, auth: Auth) -> bool {
    if principal == Principal::anonymous() {
        return false;
    }
    log!(
        INFO,
        "[{}] Authorizing `{:?}` for principal: {}",
        ic_cdk::caller(),
        auth,
        principal
    );
    mutate_state(|s| {
        let principal = PrincipalStorable(principal);
        let mut auth_set = s.auth.get(&principal).unwrap_or_default();
        if auth_set.authorize(auth) {
            s.auth.insert(principal, auth_set);
            add_metric_entry!(auths, MetricAuth(auth.to_string()), 1);
            true
        } else {
            false
        }
    })
}

pub fn do_deauthorize(principal: Principal, auth: Auth) -> bool {
    log!(
        INFO,
        "[{}] Deauthorizing `{:?}` for principal: {}",
        ic_cdk::caller(),
        auth,
        principal
    );
    mutate_state(|s| {
        let principal = PrincipalStorable(principal);
        if let Some(mut auth_set) = s.auth.get(&principal) {
            let changed = auth_set.deauthorize(auth);
            sub_metric_entry!(auths, MetricAuth(auth.to_string()), 1);
            if auth_set.is_empty() {
                s.auth.remove(&principal);
            } else {
                s.auth.insert(principal, auth_set);
            }
            changed
        } else {
            false
        }
    })
}

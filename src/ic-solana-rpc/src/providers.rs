use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_canister_log::log;
use ic_cdk::api::{is_controller, management_canister::http_request::HttpHeader};
use ic_solana::{logs::INFO, rpc_client::RpcApi};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;

use crate::{
    auth::{do_deauthorize, is_authorized, Auth},
    constants::PROVIDER_ID_MAX_SIZE,
    state::{mutate_state, read_state},
    types::{RegisterProviderArgs, RpcAuth, UpdateProviderArgs},
    utils::{hostname_from_url, validate_hostname},
};

/// Internal RPC provider representation.
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct RpcProvider {
    pub url: String,
    pub auth: Option<RpcAuth>,
    pub owner: Principal,
}

impl RpcProvider {
    pub fn api(&self) -> RpcApi {
        let mut url = self.url.clone();
        let mut headers: Option<Vec<HttpHeader>> = None;

        if let Some(auth) = &self.auth {
            match auth {
                RpcAuth::HeaderParam { name, value } => {
                    headers = Some(vec![HttpHeader {
                        name: name.clone(),
                        value: value.clone(),
                    }]);
                }
                RpcAuth::BearerToken { token } => {
                    headers = Some(vec![HttpHeader {
                        name: "Authorization".to_string(),
                        value: format!("Bearer {}", token),
                    }]);
                }
                RpcAuth::PathSegment { segment } => {
                    if !url.ends_with('/') {
                        url.push('/');
                    }
                    url.push_str(segment);
                }
                RpcAuth::QueryParam { name, value } => {
                    if url.contains('?') {
                        url.push('&');
                    } else {
                        url.push('?');
                    }
                    url.push_str(&format!("{}={}", name, value));
                }
            }
        }

        RpcApi { network: url, headers }
    }

    pub fn validate(&self) {
        match hostname_from_url(&self.url) {
            Some(hostname) => validate_hostname(&hostname).unwrap(),
            None => {
                ic_cdk::trap(&format!("Invalid RPC URL: {}", self.url));
            }
        }
    }
}

impl Storable for RpcProvider {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1000,
        is_fixed_size: false,
    };
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProviderId(pub String);

impl ProviderId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Storable for ProviderId {
    fn to_bytes(&self) -> Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: PROVIDER_ID_MAX_SIZE,
        is_fixed_size: false,
    };
}

pub fn find_provider(id: &str) -> Option<RpcProvider> {
    read_state(|s| s.rpc_providers.get(&ProviderId::new(id)))
}

/// Registers provider.
pub fn do_register_provider(caller: Principal, args: RegisterProviderArgs) {
    let provider = RpcProvider {
        url: args.url,
        auth: args.auth,
        owner: caller,
    };
    provider.validate();
    do_deauthorize(caller, Auth::RegisterProvider);
    log!(INFO, "[{}] Registering provider: {:?}", caller, args.id);
    mutate_state(|s| {
        let id = ProviderId::new(args.id);
        if s.rpc_providers.contains_key(&id) {
            ic_cdk::trap("Provider already exists");
        }
        s.rpc_providers.insert(id, provider)
    });
}

/// Unregister provider. The caller must be the owner or administrator.
pub fn do_unregister_provider(caller: Principal, provider_id: &str) -> bool {
    let is_manager = is_authorized(&caller, Auth::Manage);
    mutate_state(|s| {
        let id = ProviderId::new(provider_id);
        if let Some(provider) = s.rpc_providers.get(&id) {
            if provider.owner == caller || is_controller(&caller) || is_manager {
                log!(INFO, "[{}] Unregistering provider: {:?}", caller, provider_id);
                s.rpc_providers.remove(&id).is_some()
            } else {
                ic_cdk::trap("Unauthorized");
            }
        } else {
            false
        }
    })
}

/// Change provider details. The caller must be the owner or administrator.
pub fn do_update_provider(caller: Principal, args: UpdateProviderArgs) {
    let provider_id = ProviderId::new(args.id);
    let is_manager = is_authorized(&caller, Auth::Manage);
    mutate_state(|s| match s.rpc_providers.get(&provider_id) {
        Some(mut provider) => {
            if provider.owner == caller {
                if args.url.is_some() {
                    ic_cdk::trap("You are not authorized to update the `url` field");
                }
                if let Some(auth) = args.auth {
                    provider.auth = Some(auth);
                }
                s.rpc_providers.insert(provider_id, provider);
            } else if is_controller(&caller) || is_manager {
                if let Some(url) = args.url {
                    provider.url = url;
                }
                if let Some(auth) = args.auth {
                    provider.auth = Some(auth);
                }
                s.rpc_providers.insert(provider_id, provider);
            } else {
                ic_cdk::trap("Unauthorized");
            }
        }
        None => ic_cdk::trap("Provider not found"),
    });
}

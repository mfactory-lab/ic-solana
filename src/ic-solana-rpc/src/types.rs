use std::borrow::Cow;

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrincipalStorable(pub Principal);

impl Storable for PrincipalStorable {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::from(self.0.as_slice())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(Principal::from_slice(&bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: Principal::MAX_LENGTH_IN_BYTES as u32,
        is_fixed_size: false,
    };
}

#[derive(Debug, CandidType, Deserialize, Serialize)]
pub struct SendTransactionRequest {
    pub instructions: Vec<String>,
    pub recent_blockhash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub enum RpcAuth {
    BearerToken { token: String },
    PathSegment { segment: String },
    HeaderParam { name: String, value: String },
    QueryParam { name: String, value: String },
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RegisterProviderArgs {
    /// Unique identifier for the provider
    pub id: String,
    /// URL of the RPC endpoint
    pub url: String,
    /// Optional authentication
    pub auth: Option<RpcAuth>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct UpdateProviderArgs {
    /// The id of the provider to update
    pub id: String,
    /// URL of the RPC endpoint
    pub url: Option<String>,
    /// Optional authentication
    pub auth: Option<RpcAuth>,
}

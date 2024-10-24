use {
    crate::types::Cluster,
    candid::{CandidType, Deserialize},
    ic_cdk::api::{
        call::RejectionCode,
        management_canister::http_request::{CanisterHttpRequestArgument, HttpHeader},
    },
    serde::Serialize,
    std::{fmt::Debug, str::FromStr},
    thiserror::Error,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Deserialize, CandidType)]
pub enum ConsensusStrategy {
    /// All providers must return the same non-error result.
    #[default]
    Equality,

    /// A subset of providers must return the same non-error result.
    Threshold(u8),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

impl<T> From<JsonRpcResponse<T>> for RpcResult<T> {
    fn from(response: JsonRpcResponse<T>) -> Self {
        match (response.result, response.error) {
            (Some(result), _) => Ok(result),
            (None, Some(error)) => Err(error.into()),
            (None, None) => Err(RpcError::Text(
                "Empty response: both result and error are None".to_string(),
            )),
        }
    }
}

pub type RpcResult<T> = anyhow::Result<T, RpcError>;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, CandidType, Deserialize)]
pub struct RpcApi {
    pub network: String,
    pub headers: Option<Vec<HttpHeader>>,
}

impl RpcApi {
    pub fn cluster(&self) -> Cluster {
        Cluster::from_str(&self.network).expect("Failed to parse cluster url")
    }
}

impl Debug for RpcApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let host = self.cluster().host_str().unwrap_or("N/A".to_string());
        write!(f, "RpcApi {{ host: {} }}", host) // URL or header value could contain API keys
    }
}

pub type RequestCostCalculator = fn(&CanisterHttpRequestArgument) -> (u128, u128);
pub type HostValidator = fn(&str) -> bool;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Error, Deserialize, CandidType)]
pub enum RpcError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("HTTP outcall error: (code: {code:?}): {message}")]
    HttpOutcallError { code: RejectionCode, message: String },

    #[error("JSON-RPC error: {0}")]
    JsonRpcError(JsonRpcError),

    #[error("Parse error: expected {0}")]
    ParseError(String),

    #[error("Inconsistent response: {0:?}")]
    InconsistentResponse(Vec<(RpcApi, String)>),

    #[error("{0}")]
    Text(String),
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, CandidType, Serialize, Deserialize, Error)]
#[error("JSON-RPC error (code: {code}): {message}")]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl From<(RejectionCode, String)> for RpcError {
    fn from((code, message): (RejectionCode, String)) -> Self {
        RpcError::HttpOutcallError { code, message }
    }
}

impl From<JsonRpcError> for RpcError {
    fn from(err: JsonRpcError) -> Self {
        RpcError::JsonRpcError(err)
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(e: serde_json::Error) -> Self {
        RpcError::ParseError(e.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum RpcServices {
    Provider(Vec<String>),
    Custom(Vec<RpcApi>),
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, CandidType, Deserialize)]
pub enum RpcService {
    Provider(String),
    Custom(RpcApi),
}

#[derive(Clone, Debug, PartialEq, Eq, Default, CandidType, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcConfig {
    pub response_size_estimate: Option<u64>,
    pub response_consensus: Option<ConsensusStrategy>,
}

use crate::cluster::Cluster;
use crate::commitment::CommitmentConfig;
use crate::config::{RpcAccountInfoConfig, RpcSupplyConfig};
use crate::constants::PDA_ACCOUNT_MAX_SIZE;
use crate::logs::{DEBUG, TRACE_HTTP};
use crate::request::RpcRequest;
use crate::response::{EncodedConfirmedBlock, OptionalContext, RpcSupply};
use crate::state::{mutate_state, State};
use crate::types::account::Account;
use crate::types::epoch_info::EpochInfo;
use crate::types::transaction::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use crate::types::Slot;
use crate::utils::get_http_request_cost;
use anyhow::Result;
use candid::CandidType;
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;

/// The maximum size of an RPC response header in bytes.
pub const HEADER_SIZE_LIMIT: u64 = 1 * 1024;

#[derive(Debug, Deserialize, Clone)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, thiserror::Error, CandidType)]
pub enum RpcError {
    #[error("RPC request error: {0}")]
    RpcRequestError(String),
    #[error("RPC response error {code}: {message} {data:?}")]
    RpcResponseError {
        code: i64,
        message: String,
        data: Option<String>,
    },
    #[error("parse error: expected {0}")]
    ParseError(String),
    #[error("{0}")]
    Text(String),
}

impl From<JsonRpcError> for RpcError {
    fn from(e: JsonRpcError) -> Self {
        Self::RpcResponseError {
            code: e.code,
            message: e.message,
            data: None,
        }
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(e: serde_json::Error) -> Self {
        let error_string = e.to_string();
        Self::ParseError(error_string)
    }
}

pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpcClient {
    pub cluster: Cluster,
    pub commitment_config: CommitmentConfig,
}

impl RpcClient {
    pub fn new(cluster: String) -> Self {
        Self {
            cluster: Cluster::from_str(&cluster).unwrap(),
            commitment_config: Default::default(),
        }
    }

    pub fn from_state(state: &State) -> Self {
        Self {
            cluster: Cluster::from_str(&state.rpc_url).unwrap(),
            commitment_config: Default::default(),
        }
    }

    pub fn with_commitment(mut self, commitment_config: CommitmentConfig) -> Self {
        self.commitment_config = commitment_config;
        self
    }

    pub async fn call(&self, payload: &str, max_response_bytes: u64) -> RpcResult<String> {
        let request = CanisterHttpRequestArgument {
            url: self.cluster.url().to_string(),
            max_response_bytes: Some(max_response_bytes + HEADER_SIZE_LIMIT),
            method: HttpMethod::POST,
            headers: vec![HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            }],
            body: Some(payload.as_bytes().to_vec()),
            transform: Some(TransformContext::from_name(
                "cleanup_response".to_owned(),
                vec![],
            )),
        };

        let url = self.cluster.url();
        let cycles = get_http_request_cost(
            url,
            payload.len() as u64,
            max_response_bytes + HEADER_SIZE_LIMIT,
        );

        ic_cdk::println!("call cycles: {cycles}");

        log!(DEBUG, "Calling url: {url}, with payload: {payload}");

        match http_request(request, cycles).await {
            Ok((response,)) => {
                log!(
                    TRACE_HTTP,
                    "Got response (with {} bytes): {} from url: {} with status: {}",
                    response.body.len(),
                    String::from_utf8_lossy(&response.body),
                    url,
                    response.status
                );

                match String::from_utf8(response.body) {
                    Ok(body) => Ok(body),
                    Err(error) => Err(RpcError::ParseError(error.to_string())),
                }
            }
            Err((r, m)) => Err(RpcError::RpcRequestError(format!("({r:?}) {m:?}"))),
        }
    }

    ///
    /// Returns the lamport balance of the account of provided Pubkey.
    ///
    pub async fn get_balance(
        &self,
        pubkey: &str,
        commitment: Option<CommitmentConfig>,
    ) -> RpcResult<u64> {
        let payload = RpcRequest::GetBalance
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([pubkey, {
                    "commitment": commitment
                }]),
            )
            .to_string();

        let response = self.call(&payload, 156).await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<OptionalContext<u64>>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap().parse_value())
        }
    }

    ///
    /// Returns all information associated with the account of provided Pubkey.
    ///
    pub async fn get_account_info(
        &self,
        pubkey: &str,
        config: RpcAccountInfoConfig,
        max_response_bytes: Option<u64>,
    ) -> RpcResult<Option<Account>> {
        let payload = RpcRequest::GetAccountInfo
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([pubkey, config]),
            )
            .to_string();

        let response = self
            .call(&payload, max_response_bytes.unwrap_or(PDA_ACCOUNT_MAX_SIZE))
            .await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<OptionalContext<Account>>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(Option::from(json_response.result.unwrap().parse_value()))
        }
    }

    ///
    /// Returns the current health of the node.
    /// A healthy node is one that is within HEALTH_CHECK_SLOT_DISTANCE slots of the latest cluster confirmed slot.
    ///
    pub async fn get_health(&self) -> RpcResult<String> {
        let payload = RpcRequest::GetHealth
            .build_request_json(mutate_state(State::next_request_id), Value::Null)
            .to_string();

        let response = self.call(&payload, 256).await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<String>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns identity and transaction information about a confirmed block in the ledger.
    ///
    pub async fn get_block(
        &self,
        slot: Slot,
        encoding: UiTransactionEncoding,
    ) -> RpcResult<EncodedConfirmedBlock> {
        let payload = RpcRequest::GetBlock
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([slot, encoding]),
            )
            .to_string();

        // Estimated Size Calculation
        // String fields:
        // previous_blockhash: 46 bytes
        // blockhash: 46 bytes
        // parent_slot: 8 bytes
        // transactions:
        //   512 bytes/transaction × 1000 transactions = 512000 bytes
        // rewards: 256 bytes
        // block_time: 9 bytes (8 bytes for u64 + 1 byte for Option tag)
        // block_height: 9 bytes (8 bytes for u64 + 1 byte for Option tag)
        // Total = 92 + 8 + 512,000 + 256 + 9 + 9 = 512,374 bytes
        let response = self.call(&payload, 512374).await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<EncodedConfirmedBlock>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns information about the current supply.
    ///
    pub async fn get_supply(&self, config: RpcSupplyConfig) -> RpcResult<RpcSupply> {
        let payload = RpcRequest::GetSupply
            .build_request_json(mutate_state(State::next_request_id), json!([config]))
            .to_string();

        let response = self.call(&payload, 1024).await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<RpcSupply>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns information about the current epoch.
    ///
    pub async fn get_epoch_info(&self, commitment: CommitmentConfig) -> RpcResult<EpochInfo> {
        let payload = RpcRequest::GetEpochInfo
            .build_request_json(mutate_state(State::next_request_id), json!([commitment]))
            .to_string();

        let response = self.call(&payload, 50).await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<EpochInfo>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns transaction details for a confirmed transaction.
    ///
    pub async fn get_transaction(
        &self,
        signature: &str,
        commitment: Option<CommitmentConfig>,
        max_response_bytes: u64,
    ) -> RpcResult<EncodedConfirmedTransactionWithStatusMeta> {
        let payload = RpcRequest::GetTransaction
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([
                    signature,
                    {
                        "commitment": commitment,
                    },
                ]),
            )
            .to_string();

        let response = self.call(&payload, max_response_bytes).await?;

        let json_response = serde_json::from_str::<
            JsonRpcResponse<EncodedConfirmedTransactionWithStatusMeta>,
        >(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    // pub async fn send_transaction(
    //     &self,
    //     transaction: &impl SerializableTransaction,
    // ) -> RpcResult<String> {
    //     if (recentBlockHash == null) {
    //         recentBlockHash = getRecentBlockhash();
    //     }
    //     todo!()
    // }
}

// pub fn send_and_confirm_transaction(
//     transaction: &impl SerializableTransaction,
// ) -> ClientResult<Signature> {
//     self.invoke((self.rpc_client.as_ref()).send_and_confirm_transaction(transaction))
// }
//
// pub fn send_transaction(transaction: &impl SerializableTransaction) -> ClientResult<Signature> {
//     self.invoke((self.rpc_client.as_ref()).send_transaction(transaction))
// }

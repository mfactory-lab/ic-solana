use crate::cluster::Cluster;
use crate::constants::*;
use crate::logs::{DEBUG, TRACE_HTTP};
use crate::request::RpcRequest;
use crate::response::{
    EncodedConfirmedBlock, OptionalContext, RpcBlockhash,
    RpcConfirmedTransactionStatusWithSignature, RpcKeyedAccount, RpcSupply, RpcVersionInfo,
};
use crate::state::{mutate_state, State};
use crate::types::account::{Account, UiTokenAmount};
use crate::types::blockhash::BlockHash;
use crate::types::commitment::CommitmentConfig;
use crate::types::config::{
    RpcAccountInfoConfig, RpcContextConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig,
    RpcSignatureStatusConfig, RpcSignaturesForAddressConfig, RpcSupplyConfig,
};
use crate::types::epoch_info::EpochInfo;
use crate::types::pubkey::Pubkey;
use crate::types::signature::Signature;
use crate::types::transaction::{
    EncodedConfirmedTransactionWithStatusMeta, Transaction, TransactionStatus,
    UiTransactionEncoding,
};
use crate::types::Slot;
use crate::utils::get_http_request_cost;
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use candid::CandidType;
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;

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
    pub header_limit_size: u64,
}

impl RpcClient {
    pub fn new(cluster: &str) -> Self {
        Self {
            cluster: Cluster::from_str(cluster).unwrap(),
            commitment_config: Default::default(),
            header_limit_size: DEFAULT_HEADER_SIZE_LIMIT,
        }
    }

    pub fn from_state(state: &State) -> Self {
        Self::new(&state.rpc_url)
    }

    pub fn with_commitment(mut self, commitment_config: CommitmentConfig) -> Self {
        self.commitment_config = commitment_config;
        self
    }

    pub fn with_header_limit_size(mut self, header_limit_size: u64) -> Self {
        self.header_limit_size = header_limit_size;
        self
    }

    /// Asynchronously sends an HTTP POST request to the specified URL with the given payload and
    /// maximum response bytes, and returns the response as a string.
    /// This function calculates the required cycles for the HTTP request and logs the request
    /// details and response status. It uses a transformation named "cleanup_response" for the
    /// response body.
    ///
    /// # Arguments
    ///
    /// * `payload` - A string slice that holds the JSON payload to be sent in the HTTP request.
    /// * `max_response_bytes` - A u64 value representing the maximum number of bytes for the response.
    ///
    /// # Returns
    ///
    /// * `RpcResult<String>` - A result type that contains the response body as a string if the request
    /// is successful, or an `RpcError` if the request fails.
    ///
    /// # Errors
    ///
    /// This function returns an `RpcError` in the following cases:
    /// * If the response body cannot be parsed as a UTF-8 string, a `ParseError` is returned.
    /// * If the HTTP request fails, an `RpcRequestError` is returned with the error details.
    ///
    pub async fn call(&self, payload: &str, max_response_bytes: u64) -> RpcResult<String> {
        let max_response_bytes = max_response_bytes + self.header_limit_size;

        let request = CanisterHttpRequestArgument {
            url: self.cluster.url().to_string(),
            max_response_bytes: Some(max_response_bytes),
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
        let cycles = get_http_request_cost(url, payload.len() as u64, max_response_bytes);

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
    /// Returns the latest blockhash.
    ///
    /// Method relies on the `getLatestBlockhash` RPC call to get the latest blockhash:
    ///   https://solana.com/docs/rpc/http/getLatestBlockhash
    ///
    pub async fn get_latest_blockhash(&self, config: RpcContextConfig) -> RpcResult<BlockHash> {
        let payload = RpcRequest::GetLatestBlockhash
            .build_request_json(mutate_state(State::next_request_id), json!([config]))
            .to_string();

        let response = self.call(&payload, 156).await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<OptionalContext<RpcBlockhash>>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            let RpcBlockhash {
                blockhash,
                last_valid_block_height: _,
            } = json_response.result.unwrap().parse_value();

            let blockhash = blockhash
                .parse()
                .map_err(|_| RpcError::ParseError("BlockHash".to_string()))?;

            Ok(blockhash)
        }
    }

    ///
    /// Returns the lamport balance of the account of provided Pubkey.
    ///
    /// Method relies on the `getBalance` RPC call to get the balance:
    ///   https://solana.com/docs/rpc/http/getBalance
    ///
    pub async fn get_balance(&self, pubkey: &Pubkey, config: RpcContextConfig) -> RpcResult<u64> {
        let payload = RpcRequest::GetBalance
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([pubkey, config]),
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
    /// Returns the token balance of an SPL Token account.
    ///
    /// Method relies on the `getTokenAccountBalance` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenAccountBalance
    ///
    pub async fn get_token_account_balance(
        &self,
        pubkey: &Pubkey,
        commitment: Option<CommitmentConfig>,
    ) -> RpcResult<UiTokenAmount> {
        let payload = RpcRequest::GetTokenAccountBalance
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([pubkey, commitment]),
            )
            .to_string();

        let response = self.call(&payload, 256).await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<OptionalContext<UiTokenAmount>>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap().parse_value())
        }
    }

    ///
    /// Returns all information associated with the account of provided Pubkey.
    ///
    /// Method relies on the `getAccountInfo` RPC call to get the account info:
    ///   https://solana.com/docs/rpc/http/getAccountInfo
    ///
    pub async fn get_account_info(
        &self,
        pubkey: &Pubkey,
        config: RpcAccountInfoConfig,
        max_response_bytes: Option<u64>,
    ) -> RpcResult<Option<Account>> {
        let payload = RpcRequest::GetAccountInfo
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([pubkey.to_string(), config]),
            )
            .to_string();

        let response = self
            .call(
                &payload,
                max_response_bytes.unwrap_or(MAX_PDA_ACCOUNT_DATA_LENGTH),
            )
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
    /// Returns the current Solana version running on the node.
    ///
    pub async fn get_version(&self) -> RpcResult<RpcVersionInfo> {
        let payload = RpcRequest::GetVersion
            .build_request_json(mutate_state(State::next_request_id), Value::Null)
            .to_string();

        let response = self.call(&payload, 128).await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<RpcVersionInfo>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
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

        let response = self
            .call(&payload, GET_BLOCK_RESPONSE_SIZE_ESTIMATE)
            .await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<EncodedConfirmedBlock>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns the slot that has reached the given or default commitment level.
    ///
    pub async fn get_slot(&self) -> RpcResult<Slot> {
        let payload = RpcRequest::GetSlot
            .build_request_json(mutate_state(State::next_request_id), Value::Null)
            .to_string();

        let response = self.call(&payload, 128).await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<Slot>>(&response)?;

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

        let response = self.call(&payload, GET_SUPPLY_SIZE_ESTIMATE).await?;

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
    /// Method relies on the `getEpochInfo` RPC call to get the epoch info:
    ///   https://solana.com/docs/rpc/http/getEpochInfo
    ///
    pub async fn get_epoch_info(&self, config: RpcContextConfig) -> RpcResult<EpochInfo> {
        let payload = RpcRequest::GetEpochInfo
            .build_request_json(mutate_state(State::next_request_id), json!([config]))
            .to_string();

        let response = self.call(&payload, GET_EPOCH_INFO_SIZE_ESTIMATE).await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<EpochInfo>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns all accounts owned by the provided program Pubkey.
    ///
    /// Method relies on the `getProgramAccounts` RPC call to get the program accounts:
    ///   https://solana.com/docs/rpc/http/getProgramAccounts
    ///
    pub async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        config: RpcProgramAccountsConfig,
        max_response_bytes: u64,
    ) -> RpcResult<Vec<RpcKeyedAccount>> {
        let payload = RpcRequest::GetProgramAccounts
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([program_id, config]),
            )
            .to_string();

        let response = self.call(&payload, max_response_bytes).await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<Vec<RpcKeyedAccount>>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Requests an airdrop of lamports to a Pubkey
    ///
    /// Method relies on the `requestAirdrop` RPC call to request the airdrop:
    ///   https://solana.com/docs/rpc/http/requestAirdrop
    ///
    pub async fn request_airdrop(&self, pubkey: &Pubkey, lamports: u64) -> RpcResult<String> {
        let payload = RpcRequest::RequestAirdrop
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([pubkey, lamports]),
            )
            .to_string();

        let response = self.call(&payload, 156).await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<String>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns signatures for confirmed transactions that include the given address in their accountKeys list.
    /// Returns signatures backwards in time from the provided signature or the most recent confirmed block.
    ///
    /// Method relies on the `getSignaturesForAddress` RPC call to get the signatures for the address:
    ///   https://solana.com/docs/rpc/http/getsignaturesforaddress
    ///
    pub async fn get_signatures_for_address(
        &self,
        pubkey: &Pubkey,
        config: RpcSignaturesForAddressConfig,
    ) -> RpcResult<Vec<RpcConfirmedTransactionStatusWithSignature>> {
        let payload = RpcRequest::GetSignaturesForAddress
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([pubkey, config]),
            )
            .to_string();

        let max_limit = 1000;

        let response = self
            .call(
                &payload,
                SIGNATURE_RESPONSE_SIZE_ESTIMATE * config.limit.unwrap_or(max_limit) as u64,
            )
            .await?;

        let json_response = serde_json::from_str::<
            JsonRpcResponse<Vec<RpcConfirmedTransactionStatusWithSignature>>,
        >(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns the statuses of a list of transaction signatures.
    ///
    /// Method relies on the `getSignatureStatuses` RPC call to get the statuses for the signatures:
    ///   https://solana.com/docs/rpc/http/getSignatureStatuses
    ///
    pub async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
        config: RpcSignatureStatusConfig,
    ) -> RpcResult<Vec<TransactionStatus>> {
        let payload = RpcRequest::GetSignatureStatuses
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([signatures, config]),
            )
            .to_string();

        let response = self.call(&payload, 128).await?;

        let json_response =
            serde_json::from_str::<JsonRpcResponse<Vec<TransactionStatus>>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Returns transaction details for a confirmed transaction.
    ///
    /// Method relies on the `getTransaction` RPC call to get the transaction data:
    ///   https://solana.com/docs/rpc/http/getTransaction
    ///
    pub async fn get_transaction(
        &self,
        signature: &Signature,
        commitment: Option<CommitmentConfig>,
    ) -> RpcResult<EncodedConfirmedTransactionWithStatusMeta> {
        let payload = RpcRequest::GetTransaction
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([ signature, { "commitment": commitment } ]),
            )
            .to_string();

        let response = self
            .call(&payload, TRANSACTION_RESPONSE_SIZE_ESTIMATE)
            .await?;

        let json_response = serde_json::from_str::<
            JsonRpcResponse<EncodedConfirmedTransactionWithStatusMeta>,
        >(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }

    ///
    /// Submits a signed transaction to the cluster for processing.
    /// This method does not alter the transaction in any way; it relays the transaction created by clients to the node as-is.
    /// If the node's rpc service receives the transaction, this method immediately succeeds,
    /// without waiting for any confirmations.
    /// A successful response from this method does not guarantee the transaction is processed or confirmed by the cluster.
    ///
    /// Use [RpcClient::get_signature_statuses] to ensure a transaction is processed and confirmed.
    ///
    /// Method relies on the `sendTransaction` RPC call to send the transaction:
    ///   https://solana.com/docs/rpc/http/sendTransaction
    ///
    pub async fn send_transaction(
        &self,
        tx: Transaction,
        config: RpcSendTransactionConfig,
    ) -> RpcResult<Signature> {
        let serialized = tx.serialize();

        let raw_tx = match config.encoding {
            None | Some(UiTransactionEncoding::Base58) => bs58::encode(serialized).into_string(),
            Some(UiTransactionEncoding::Base64) => BASE64_STANDARD.encode(serialized),
            Some(e) => {
                return Err(RpcError::Text(format!(
                    "Unsupported encoding: {e}. Supported encodings: base58, base64"
                )));
            }
        };

        let payload = RpcRequest::SendTransaction
            .build_request_json(
                mutate_state(State::next_request_id),
                json!([raw_tx, config]),
            )
            .to_string();

        let response = self.call(&payload, 156).await?;

        let json_response = serde_json::from_str::<JsonRpcResponse<Signature>>(&response)?;

        if let Some(e) = json_response.error {
            Err(e.into())
        } else {
            Ok(json_response.result.unwrap())
        }
    }
}

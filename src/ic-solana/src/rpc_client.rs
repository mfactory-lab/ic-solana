use {
    crate::{
        constants::*,
        request::RpcRequest,
        response::{
            OptionalContext, Response, RpcBlockCommitment, RpcBlockProduction, RpcBlockhash,
            RpcConfirmedTransactionStatusWithSignature, RpcContactInfo, RpcIdentity, RpcInflationGovernor,
            RpcInflationRate, RpcInflationReward, RpcKeyedAccount, RpcPerfSample, RpcPrioritizationFee,
            RpcSnapshotSlotInfo, RpcSupply, RpcTokenAccountBalance, RpcVersionInfo, RpcVoteAccountStatus,
        },
        rpc_result::{ConsensusStrategy, MultiCallError, MultiCallResults},
        types::{
            Account, BlockHash, Cluster, CommitmentConfig, EncodedConfirmedTransactionWithStatusMeta, EpochInfo,
            EpochSchedule, Pubkey, RpcAccountInfoConfig, RpcBlockConfig, RpcBlockProductionConfig, RpcContextConfig,
            RpcEpochConfig, RpcGetVoteAccountsConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig,
            RpcSignatureStatusConfig, RpcSignaturesForAddressConfig, RpcSupplyConfig, RpcTokenAccountsFilter,
            RpcTransactionConfig, Signature, Slot, Transaction, TransactionStatus, UiAccount, UiConfirmedBlock,
            UiTokenAmount, UiTransactionEncoding,
        },
    },
    anyhow::Result,
    base64::{prelude::BASE64_STANDARD, Engine},
    candid::CandidType,
    ic_canister_log::log,
    ic_cdk::api::{
        call::RejectionCode,
        management_canister::http_request::{
            http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
        },
    },
    ic_solana_common::{
        add_metric_entry,
        logs::DEBUG,
        metrics::{MetricRpcHost, MetricRpcMethod},
    },
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    serde_json::{json, Value},
    std::{
        cell::RefCell,
        collections::{BTreeSet, HashMap},
        fmt::Debug,
        str::FromStr,
    },
    thiserror::Error,
};

thread_local! {
    static NEXT_ID: RefCell<u64> = RefCell::default();
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

impl<T> JsonRpcResponse<T> {
    fn into_rpc_result(self) -> RpcResult<T> {
        match (self.error, self.result) {
            (Some(e), _) => Err(e.into()),
            (None, Some(result)) => Ok(result),
            (None, None) => Err(RpcError::Text("Empty response".to_string())),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Error, Deserialize, CandidType)]
pub enum RpcError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("HTTP outcall error: {0}")]
    HttpOutcallError(HttpOutcallError),

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

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, CandidType, Deserialize, Error)]
pub enum HttpOutcallError {
    /// Error from the IC system API.
    #[error("IC error (code: {code:?}): {message}")]
    IcError { code: RejectionCode, message: String },
    /// Response is not a valid JSON-RPC response,
    /// which means that the response was not successful (status other than 2xx)
    /// or that the response body could not be deserialized into a JSON-RPC response.
    #[error("Invalid HTTP JSON-RPC response: status {status}, body: {body}, parsing error: {parsing_error:?}")]
    InvalidHttpJsonRpcResponse {
        status: u16,
        body: String,
        #[serde(rename = "parsingError")]
        parsing_error: Option<String>,
    },
}

impl From<HttpOutcallError> for RpcError {
    fn from(err: HttpOutcallError) -> Self {
        RpcError::HttpOutcallError(err)
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

pub type RpcResult<T> = Result<T, RpcError>;

pub type RequestCostCalculator = fn(&CanisterHttpRequestArgument) -> (u128, u128);

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RpcClientConfig {
    pub response_consensus: Option<ConsensusStrategy>,
    pub response_size_estimate: Option<u64>,
    pub cost_calculator: Option<RequestCostCalculator>,
    pub transform_context: Option<TransformContext>,
    pub is_demo_active: bool,
    pub hosts_blocklist: &'static [&'static str],
    pub extra_response_bytes: u64,
}

impl Default for RpcClientConfig {
    fn default() -> Self {
        Self {
            response_consensus: None,
            response_size_estimate: None,
            cost_calculator: None,
            transform_context: None,
            is_demo_active: false,
            hosts_blocklist: &[],
            extra_response_bytes: 2 * 1024, // 2KB
        }
    }
}

#[derive(Clone, Debug)]
pub struct RpcClient {
    pub providers: BTreeSet<RpcApi>,
    pub config: RpcClientConfig,
}

impl RpcClient {
    pub fn new<T: Into<Vec<RpcApi>>>(providers: T, config: Option<RpcClientConfig>) -> Self {
        Self {
            providers: providers.into().into_iter().collect(),
            config: config.unwrap_or_default(),
        }
    }

    fn consensus_strategy(&self) -> ConsensusStrategy {
        self.config.response_consensus.as_ref().cloned().unwrap_or_default()
    }

    ///
    /// Generate the next request id.
    ///
    pub fn next_request_id(&self) -> u64 {
        NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id = next_id.wrapping_add(1);
            id
        })
    }

    ///
    /// Asynchronously sends an HTTP POST request to the specified URL with the given payload and
    /// maximum response bytes and returns the response as a string.
    /// This function calculates the required cycles for the HTTP request and logs the request
    /// details and response status. It uses a transformation named "cleanup_response" for the
    /// response body.
    ///
    /// # Arguments
    ///
    /// * `payload` - JSON payload to be sent in the HTTP request.
    /// * `max_response_bytes` - The maximal size of the response in bytes. If None, 2MiB will be the limit.
    ///
    /// # Returns
    ///
    /// * `RpcResult<String>` - A result type that contains the response body as a string if the request
    ///   is successful, or an `RpcError` if the request fails.
    ///
    /// # Errors
    ///
    /// This function returns an `RpcError` in the following cases:
    /// * If the response body cannot be parsed as a UTF-8 string, a `ParseError` is returned.
    /// * If the HTTP request fails, an `RpcRequestError` is returned with the error details.
    ///
    async fn call_internal(
        &self,
        provider: &RpcApi,
        payload: &Value,
        max_response_bytes: Option<u64>,
    ) -> RpcResult<Vec<u8>> {
        let cluster = provider.cluster();
        let url = cluster.url();

        let mut headers = provider.headers.clone().unwrap_or_default();
        if !headers
            .iter()
            .any(|header| header.name.to_lowercase() == "content-type")
        {
            headers.push(HttpHeader {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            });
        }

        let body = serde_json::to_vec(payload).map_err(|e| RpcError::ParseError(e.to_string()))?;

        let request = CanisterHttpRequestArgument {
            url: url.to_string(),
            max_response_bytes: max_response_bytes.map(|n| n + self.config.extra_response_bytes),
            method: HttpMethod::POST,
            headers,
            body: Some(body),
            transform: self.config.transform_context.clone(),
        };

        // Calculate cycles if a calculator is provided
        let (cycles_cost, cycles_cost_with_collateral) = self
            .config
            .cost_calculator
            .as_ref()
            .map_or((0, 0), |calc| calc(&request));

        let parsed_url = url::Url::parse(url).map_err(|_| RpcError::ParseError(format!("Invalid URL: {}", url)))?;

        let host = parsed_url
            .host_str()
            .ok_or_else(|| RpcError::ParseError(format!("Error parsing hostname from URL: {}", url)))?;

        let rpc_host = MetricRpcHost(host.to_string());
        let rpc_method = MetricRpcMethod(Self::find_rpc_method_name(payload).to_string());

        if self.config.hosts_blocklist.contains(&host) {
            add_metric_entry!(err_host_not_allowed, rpc_host.clone(), 1);
            return Err(RpcError::Text(format!("Disallowed RPC service host: {}", host)));
        }

        // Handle cycle accounting if not in demo mode
        if !self.config.is_demo_active {
            let cycles_available = ic_cdk::api::call::msg_cycles_available128();
            if cycles_available < cycles_cost_with_collateral {
                return Err(RpcError::Text(format!(
                    "Insufficient cycles: available {}, required {} (with collateral).",
                    cycles_available, cycles_cost_with_collateral
                )));
            }
            ic_cdk::api::call::msg_cycles_accept128(cycles_cost);
            add_metric_entry!(cycles_charged, (rpc_method.clone(), rpc_host.clone()), cycles_cost);
        }

        log!(
            DEBUG,
            "Calling url: {url} with payload: {payload}. Cycles: {cycles_cost}"
        );

        add_metric_entry!(requests, (rpc_method.clone(), rpc_host.clone()), 1);

        match http_request(request, cycles_cost).await {
            Ok((response,)) => {
                let body_len = response.body.len();
                let body_str = std::str::from_utf8(&response.body).map_err(|e| RpcError::ParseError(e.to_string()))?;

                log!(
                    DEBUG,
                    "Got response (with {} bytes): {} from url: {} with status: {}",
                    body_len,
                    body_str,
                    url,
                    response.status
                );

                let status: u32 = response.status.0.try_into().unwrap_or(0);
                add_metric_entry!(responses, (rpc_method, rpc_host, status.into()), 1);

                Ok(response.body)
            }
            Err((code, message)) => {
                add_metric_entry!(err_http_outcall, (rpc_method, rpc_host), 1);
                Err(HttpOutcallError::IcError { code, message }.into())
            }
        }
    }

    async fn parallel_call(&self, payload: &Value, max_response_bytes: Option<u64>) -> Vec<RpcResult<Vec<u8>>> {
        let mut fut = Vec::with_capacity(self.providers.len());
        for provider in self.providers.iter() {
            log!(DEBUG, "[parallel_call]: will call provider: {:?}", provider);
            fut.push(async { self.call_internal(provider, payload, max_response_bytes).await });
        }
        futures::future::join_all(fut).await
    }

    ///
    /// Makes a single JSON-RPC call.
    ///
    pub async fn call<P: Serialize, R: DeserializeOwned>(
        &self,
        method: RpcRequest,
        params: P,
        max_response_bytes: Option<u64>,
    ) -> RpcResult<JsonRpcResponse<R>> {
        let payload = method.build_json(self.next_request_id(), params);
        let results = self.parallel_call(&payload, max_response_bytes).await;
        let bytes = Self::process_result(
            method,
            MultiCallResults::from_non_empty_iter(self.providers.iter().cloned().zip(results.into_iter()))
                .reduce(self.consensus_strategy()),
        )?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    ///
    /// Makes multiple JSON-RPC calls in a single batch request.
    ///
    pub async fn batch_call<P: Serialize, R: DeserializeOwned>(
        &self,
        requests: &[(RpcRequest, P)],
        max_response_bytes: Option<u64>,
    ) -> RpcResult<Vec<JsonRpcResponse<R>>> {
        let payload = RpcRequest::batch(
            requests
                .iter()
                .map(|(method, params)| (method.to_owned(), params, self.next_request_id()))
                .collect(),
        );

        let results = self.parallel_call(&payload, max_response_bytes).await;

        let bytes = Self::process_result(
            Self::find_rpc_method_name(&payload),
            MultiCallResults::from_non_empty_iter(self.providers.iter().cloned().zip(results.into_iter()))
                .reduce(self.consensus_strategy()),
        )?;

        Ok(serde_json::from_slice(&bytes)?)
    }

    ///
    /// Returns the latest blockhash.
    ///
    /// Method relies on the `getLatestBlockhash` RPC call to get the latest blockhash:
    ///   https://solana.com/docs/rpc/http/getLatestBlockhash
    ///
    pub async fn get_latest_blockhash(&self, config: Option<RpcContextConfig>) -> RpcResult<BlockHash> {
        let response = self
            .call::<_, OptionalContext<RpcBlockhash>>(RpcRequest::GetLatestBlockhash, json!([config]), Some(156))
            .await?;

        let blockhash = response.into_rpc_result()?.parse_value();

        let blockhash = blockhash
            .blockhash
            .parse()
            .map_err(|_| RpcError::ParseError("BlockHash".to_string()))?;

        Ok(blockhash)
    }

    ///
    /// Returns a list of recent performance samples, in reverse slot order.
    /// Performance samples are taken every 60 seconds and include the number
    /// of transactions and slots that occur in a given time window.
    ///
    /// Method relies on the `getRecentPerformanceSamples` RPC call to get the performance samples:
    ///   https://solana.com/docs/rpc/http/getRecentPerformanceSamples
    ///
    pub async fn get_recent_performance_samples(&self, limit: u64) -> RpcResult<Vec<RpcPerfSample>> {
        let response = self
            .call::<_, _>(
                RpcRequest::GetRecentPerformanceSamples,
                json!([limit]),
                Some(256 * limit),
            )
            .await?;

        response.into_rpc_result()
    }

    ///
    /// Returns a list of prioritization fees from recent blocks.
    ///
    /// Method relies on the `getRecentPrioritizationFees` RPC call to get the prioritization fees:
    ///   https://solana.com/docs/rpc/http/getRecentPrioritizationFees
    ///
    pub async fn get_recent_prioritization_fees(&self, addresses: &[Pubkey]) -> RpcResult<Vec<RpcPrioritizationFee>> {
        let response = self
            .call::<_, _>(
                RpcRequest::GetRecentPrioritizationFees,
                json!([addresses]),
                Some(128 * addresses.len() as u64),
            )
            .await?;

        response.into_rpc_result()
    }

    ///
    /// Returns the lamport balance of the account of provided Pubkey.
    ///
    /// Method relies on the `getBalance` RPC call to get the balance:
    ///   https://solana.com/docs/rpc/http/getBalance
    ///
    pub async fn get_balance(&self, pubkey: &Pubkey, config: Option<RpcContextConfig>) -> RpcResult<u64> {
        let response = self
            .call::<_, OptionalContext<u64>>(RpcRequest::GetBalance, json!([pubkey.to_string(), config]), Some(156))
            .await?;

        response.into_rpc_result().map(|c| c.parse_value())
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
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<UiTokenAmount> {
        let response = self
            .call::<_, OptionalContext<UiTokenAmount>>(
                RpcRequest::GetTokenAccountBalance,
                json!([pubkey.to_string(), commitment_config]),
                Some(256),
            )
            .await?;

        response.into_rpc_result().map(|c| c.parse_value())
    }

    ///
    /// Returns all SPL Token accounts by approved Delegate.
    ///
    /// Method relies on the `getTokenAccountsByDelegate` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenAccountsByDelegate
    ///
    pub async fn get_token_accounts_by_delegate(
        &self,
        pubkey: &Pubkey,
        filter: RpcTokenAccountsFilter,
        config: Option<RpcAccountInfoConfig>,
    ) -> RpcResult<Vec<RpcKeyedAccount>> {
        let response = self
            .call::<_, OptionalContext<Vec<RpcKeyedAccount>>>(
                RpcRequest::GetTokenAccountsByDelegate,
                json!([pubkey.to_string(), filter, config]),
                self.config
                    .response_size_estimate
                    .or(Some(GET_TOKEN_ACCOUNTS_SIZE_ESTIMATE)),
            )
            .await?;

        response.into_rpc_result().map(|c| c.parse_value())
    }

    ///
    /// Returns all SPL Token accounts by token owner.
    ///
    /// Method relies on the `getTokenAccountsByOwner` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenAccountsByOwner
    ///
    pub async fn get_token_accounts_by_owner(
        &self,
        pubkey: &Pubkey,
        filter: RpcTokenAccountsFilter,
        config: Option<RpcAccountInfoConfig>,
    ) -> RpcResult<Vec<RpcKeyedAccount>> {
        let response = self
            .call::<_, OptionalContext<Vec<RpcKeyedAccount>>>(
                RpcRequest::GetTokenAccountsByOwner,
                json!([pubkey.to_string(), filter, config]),
                self.config
                    .response_size_estimate
                    .or(Some(GET_TOKEN_ACCOUNTS_SIZE_ESTIMATE)),
            )
            .await?;

        response.into_rpc_result().map(|c| c.parse_value())
    }

    ///
    /// Returns the 20 largest accounts of a particular SPL Token type.
    ///
    /// Method relies on the `getTokenLargestAccounts` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenLargestAccounts
    ///
    pub async fn get_token_largest_accounts(
        &self,
        mint: &Pubkey,
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<Vec<RpcTokenAccountBalance>> {
        let response = self
            .call::<_, OptionalContext<Vec<RpcTokenAccountBalance>>>(
                RpcRequest::GetTokenLargestAccounts,
                json!([mint.to_string(), commitment_config]),
                self.config
                    .response_size_estimate
                    .or(Some(GET_TOKEN_LARGEST_ACCOUNTS_SIZE_ESTIMATE)),
            )
            .await?;

        response.into_rpc_result().map(|c| c.parse_value())
    }

    ///
    /// Returns the total supply of an SPL Token type.
    ///
    /// Method relies on the `getTokenSupply` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenSupply
    ///
    pub async fn get_token_supply(
        &self,
        mint: &Pubkey,
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<UiTokenAmount> {
        let response = self
            .call::<_, OptionalContext<UiTokenAmount>>(
                RpcRequest::GetTokenSupply,
                json!([mint.to_string(), commitment_config]),
                self.config
                    .response_size_estimate
                    .or(Some(GET_TOKEN_SUPPLY_SIZE_ESTIMATE)),
            )
            .await?;

        response.into_rpc_result().map(|c| c.parse_value())
    }

    ///
    /// Returns all information associated with the account of the provided Pubkey.
    ///
    /// Method relies on the `getAccountInfo` RPC call to get the account info:
    ///   https://solana.com/docs/rpc/http/getAccountInfo
    ///
    pub async fn get_account_info(
        &self,
        pubkey: &Pubkey,
        config: Option<RpcAccountInfoConfig>,
    ) -> RpcResult<Option<Account>> {
        let response = self
            .call::<_, Response<Option<UiAccount>>>(
                RpcRequest::GetAccountInfo,
                json!([pubkey.to_string(), config]),
                self.config.response_size_estimate.or(Some(MAX_PDA_ACCOUNT_DATA_LENGTH)),
            )
            .await?;

        let response = response.into_rpc_result()?;

        let account = response
            .value
            .ok_or_else(|| RpcError::Text(format!("Account not found: {}", pubkey)))?;

        Ok(account.decode())
    }

    ///
    /// Returns the current Solana version running on the node.
    ///
    /// Method relies on the `getVersion` RPC call to get the version info:
    ///   https://solana.com/docs/rpc/http/getVersion
    ///
    pub async fn get_version(&self) -> RpcResult<RpcVersionInfo> {
        self.call::<_, RpcVersionInfo>(RpcRequest::GetVersion, (), Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the account info and associated stake for all the voting accounts in the current bank.
    ///
    /// Method relies on the `getVoteAccounts` RPC call to get the voting accounts:
    ///   https://solana.com/docs/rpc/http/getVoteAccounts
    ///
    pub async fn get_vote_accounts(&self, config: RpcGetVoteAccountsConfig) -> RpcResult<RpcVoteAccountStatus> {
        self.call::<_, RpcVoteAccountStatus>(
            RpcRequest::GetVoteAccounts,
            [config],
            Some(GET_VOTE_ACCOUNTS_SIZE_ESTIMATE),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns the current health of the node.
    /// A healthy node is one that is within HEALTH_CHECK_SLOT_DISTANCE slots of the latest cluster-confirmed slot.
    ///
    /// Method relies on the `getHealth` RPC call to get the health status:
    ///   https://solana.com/docs/rpc/http/getHealth
    ///
    pub async fn get_health(&self) -> RpcResult<String> {
        self.call::<_, String>(RpcRequest::GetHealth, (), Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the highest slot information that the node has snapshots for.
    /// This will find the highest full snapshot slot and the highest incremental
    /// snapshot slot based on the full snapshot slot, if there is one.
    ///
    /// Method relies on the `getHighestSnapshotSlot` RPC call to get the highest snapshot slot:
    ///   https://solana.com/docs/rpc/http/getHighestSnapshotSlot
    ///
    pub async fn get_highest_snapshot_slot(&self) -> RpcResult<RpcSnapshotSlotInfo> {
        self.call::<_, RpcSnapshotSlotInfo>(RpcRequest::GetHighestSnapshotSlot, (), Some(256))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the identity pubkey for the current node.
    ///
    /// Method relies on the `getIdentity` RPC call to get the identity pubkey:
    ///   https://solana.com/docs/rpc/http/getIdentity
    ///
    pub async fn get_identity(&self) -> RpcResult<RpcIdentity> {
        self.call::<_, RpcIdentity>(RpcRequest::GetIdentity, (), Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the current inflation governor.
    ///
    /// Method relies on the `getInflationGovernor` RPC call to get inflation governor:
    ///   https://solana.com/docs/rpc/http/getInflationGovernor
    ///
    pub async fn get_inflation_governor(&self) -> RpcResult<RpcInflationGovernor> {
        self.call::<_, RpcInflationGovernor>(RpcRequest::GetInflationGovernor, (), Some(256))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the specific inflation values for the current epoch.
    ///
    /// Method relies on the `getInflationRate` RPC call to get inflation rate:
    ///   https://solana.com/docs/rpc/http/getInflationRate
    ///
    pub async fn get_inflation_rate(&self) -> RpcResult<RpcInflationRate> {
        self.call::<_, RpcInflationRate>(RpcRequest::GetInflationRate, (), Some(256))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the inflation / staking reward for a list of addresses for an epoch.
    ///
    /// Method relies on the `getInflationReward` RPC call to get inflation reward:
    ///   https://solana.com/docs/rpc/http/getInflationReward
    ///
    pub async fn get_inflation_reward(
        &self,
        addresses: &[Pubkey],
        config: RpcEpochConfig,
    ) -> RpcResult<Vec<Option<RpcInflationReward>>> {
        self.call::<_, Vec<Option<RpcInflationReward>>>(
            RpcRequest::GetInflationReward,
            json!([addresses, config]),
            Some(40 + 153 * addresses.len() as u64),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns identity and transaction information about a confirmed block in the ledger.
    ///
    /// Method relies on the `getBlock` RPC call to get the block:
    ///   https://solana.com/docs/rpc/http/getBlock
    ///
    pub async fn get_block(&self, slot: Slot, config: Option<RpcBlockConfig>) -> RpcResult<UiConfirmedBlock> {
        self.call::<_, UiConfirmedBlock>(
            RpcRequest::GetBlock,
            json!([slot, config]),
            self.config.response_size_estimate.or(Some(GET_BLOCK_SIZE_ESTIMATE)),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns commitment for a particular block.
    ///
    /// Method relies on the `getBlockCommitment` RPC call to get the block commitment:
    ///   https://solana.com/docs/rpc/http/getBlockCommitment
    ///
    pub async fn get_block_commitment(&self, slot: Slot) -> RpcResult<RpcBlockCommitment> {
        self.call::<_, RpcBlockCommitment>(
            RpcRequest::GetBlockCommitment,
            json!([slot]),
            Some(GET_BLOCK_COMMITMENT_SIZE_ESTIMATE),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns a list of confirmed blocks between two slots.
    ///
    /// Method relies on the `getBlocks` RPC call to get the blocks:
    ///   https://solana.com/docs/rpc/http/getBlocks
    ///
    pub async fn get_blocks(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<Vec<u64>> {
        let params = if end_slot.is_some() {
            json!([start_slot, end_slot, commitment_config])
        } else {
            json!([start_slot, commitment_config])
        };

        // Total response size estimation
        let end_slot = end_slot.unwrap_or(start_slot + MAX_GET_BLOCKS_RANGE);
        let limit = end_slot.saturating_sub(start_slot);

        if limit > MAX_GET_BLOCKS_RANGE {
            return Err(RpcError::ValidationError(format!(
                "Slot range too large; must be less or equal than {}",
                MAX_GET_BLOCKS_RANGE
            )));
        }

        self.call::<_, Vec<u64>>(
            RpcRequest::GetBlocks,
            params,
            Some(Self::get_block_range_max_response_bytes(start_slot, limit)),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns a list of confirmed blocks starting at the given slot.
    ///
    /// Method relies on the `getBlocksWithLimit` RPC call to get the blocks with limit:
    ///   https://solana.com/docs/rpc/http/getBlocksWithLimit
    ///
    pub async fn get_blocks_with_limit(
        &self,
        start_slot: Slot,
        limit: u64,
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<Vec<u64>> {
        if limit > MAX_GET_BLOCKS_RANGE {
            return Err(RpcError::ValidationError(format!(
                "Limit too large, must be less or equal than {}",
                MAX_GET_BLOCKS_RANGE
            )));
        }

        self.call::<_, Vec<u64>>(
            RpcRequest::GetBlocksWithLimit,
            json!([start_slot, limit, commitment_config]),
            Some(Self::get_block_range_max_response_bytes(start_slot, limit)),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns the current block height of the node.
    ///
    /// Method relies on the `getBlockHeight` RPC call to get the block height:
    ///   https://solana.com/docs/rpc/http/getBlockHeight
    ///
    pub async fn get_block_height(&self, config: Option<RpcContextConfig>) -> RpcResult<u64> {
        self.call::<_, u64>(
            RpcRequest::GetBlockHeight,
            json!([config.unwrap_or_default()]),
            Some(45),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns the estimated production time of a block.
    ///
    /// Method relies on the `getBlockTime` RPC call to get the block time:
    ///   https://solana.com/docs/rpc/http/getBlockTime
    ///
    pub async fn get_block_time(&self, slot: Slot) -> RpcResult<i64> {
        self.call::<_, i64>(RpcRequest::GetBlockTime, json!([slot]), Some(45))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns recent block production information from the current or previous epoch.
    ///
    /// Method relies on the `getBlockProduction` RPC call to get the block production:
    ///   https://solana.com/docs/rpc/http/getBlockProduction
    ///
    pub async fn get_block_production(&self, config: RpcBlockProductionConfig) -> RpcResult<RpcBlockProduction> {
        self.call::<_, OptionalContext<RpcBlockProduction>>(
            RpcRequest::GetBlockProduction,
            json!([config]),
            Some(GET_BLOCK_PRODUCTION_SIZE_ESTIMATE),
        )
        .await?
        .into_rpc_result()
        .map(|c| c.parse_value())
    }

    ///
    /// Returns information about all the nodes participating in the cluster
    ///
    /// Method relies on the `getClusterNodes` RPC call to get the cluster nodes:
    ///   https://solana.com/docs/rpc/http/getClusterNodes
    ///
    pub async fn get_cluster_nodes(&self) -> RpcResult<Vec<RpcContactInfo>> {
        self.call::<_, Vec<RpcContactInfo>>(RpcRequest::GetClusterNodes, (), None)
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the slot that has reached the given or default commitment level.
    ///
    /// Method relies on the `getSlot` RPC call to get the slot:
    ///   https://solana.com/docs/rpc/http/getSlot
    ///
    pub async fn get_slot(&self, config: Option<RpcContextConfig>) -> RpcResult<Slot> {
        self.call::<_, Slot>(RpcRequest::GetSlot, json!([config]), Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the current slot leader.
    ///
    /// Method relies on the `getSlotLeader` RPC call to get the slot leader:
    ///   https://solana.com/docs/rpc/http/getSlotLeader
    ///
    pub async fn get_slot_leader(&self, config: Option<RpcContextConfig>) -> RpcResult<String> {
        self.call::<_, String>(RpcRequest::GetSlotLeader, json!([config]), Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the slot leaders for a given slot range.
    ///
    /// Method relies on the `getSlotLeaders` RPC call to get the slot leaders:
    ///   https://solana.com/docs/rpc/http/getSlotLeaders
    ///
    pub async fn get_slot_leaders(&self, start_slot: u64, limit: Option<u64>) -> RpcResult<String> {
        let limit = limit.unwrap_or(MAX_GET_SLOT_LEADERS);

        if limit > MAX_GET_SLOT_LEADERS {
            return Err(RpcError::ValidationError(format!(
                "Exceeded maximum limit of {}",
                MAX_GET_SLOT_LEADERS
            )));
        }

        let commas_size = if limit > 0 { limit - 1 } else { 0 };
        let max_response_bytes = 36 + 46 * limit + commas_size;

        self.call::<_, String>(
            RpcRequest::GetSlotLeaders,
            json!([start_slot, limit]),
            Some(max_response_bytes),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns the stake minimum delegation, in lamports.
    ///
    /// Method relies on the `getStakeMinimumDelegation` RPC call to get the supply:
    ///   https://solana.com/docs/rpc/http/getStakeMinimumDelegation
    ///
    pub async fn get_stake_minimum_delegation(&self, config: Option<CommitmentConfig>) -> RpcResult<Response<u64>> {
        self.call::<_, Response<u64>>(RpcRequest::GetStakeMinimumDelegation, json!([config]), Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns information about the current supply.
    ///
    /// Method relies on the `getSupply` RPC call to get the supply:
    ///   https://solana.com/docs/rpc/http/getSupply
    ///
    pub async fn get_supply(&self, config: RpcSupplyConfig) -> RpcResult<RpcSupply> {
        self.call::<_, RpcSupply>(RpcRequest::GetSupply, json!([config]), Some(GET_SUPPLY_SIZE_ESTIMATE))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns information about the current epoch.
    ///
    /// Method relies on the `getEpochInfo` RPC call to get the epoch info:
    ///   https://solana.com/docs/rpc/http/getEpochInfo
    ///
    pub async fn get_epoch_info(&self, config: Option<RpcContextConfig>) -> RpcResult<EpochInfo> {
        self.call::<_, EpochInfo>(
            RpcRequest::GetEpochInfo,
            json!([config]),
            Some(GET_EPOCH_INFO_SIZE_ESTIMATE),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns the epoch schedules information from this cluster's genesis config.
    ///
    /// Method relies on the `getEpochSchedule` RPC call to get the epoch schedule:
    ///   https://solana.com/docs/rpc/http/getEpochSchedule
    ///
    pub async fn get_epoch_schedule(&self) -> RpcResult<EpochSchedule> {
        self.call::<_, EpochSchedule>(
            RpcRequest::GetEpochSchedule,
            Value::Null,
            Some(GET_EPOCH_SCHEDULE_SIZE_ESTIMATE),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Get the fee the network will charge for a particular Message.
    ///
    /// Method relies on the `getFeeForMessage` RPC call to get the fee for a message:
    ///   https://solana.com/docs/rpc/http/getFeeForMessage
    ///
    pub async fn get_fee_for_message(&self, message: String, config: Option<RpcContextConfig>) -> RpcResult<u64> {
        self.call::<_, u64>(RpcRequest::GetFeeForMessage, json!([message, config]), Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the slot of the lowest confirmed block that has not been purged from the ledger.
    ///
    /// Method relies on the `getFirstAvailableBlock` RPC call to get the first available block:
    ///   https://solana.com/docs/rpc/http/getFirstAvailableBlock
    ///
    pub async fn get_first_available_block(&self) -> RpcResult<u64> {
        self.call::<_, u64>(RpcRequest::GetFirstAvailableBlock, Value::Null, Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the genesis hash.
    ///
    /// Method relies on the `getGenesisHash` RPC call to get the genesis hash:
    ///   https://solana.com/docs/rpc/http/getGenesisHash
    ///
    pub async fn get_genesis_hash(&self) -> RpcResult<String> {
        self.call::<_, String>(RpcRequest::GetGenesisHash, Value::Null, Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Returns the minimum balance required to make account rent exempt.
    ///
    /// Method relies on the `getMinimumBalanceForRentExemption` RPC call to get the minimum balance for rent exemption:
    ///   https://solana.com/docs/rpc/http/getMinimumBalanceForRentExemption
    ///
    pub async fn get_minimum_balance_for_rent_exemption(
        &self,
        data_len: usize,
        config: Option<CommitmentConfig>,
    ) -> RpcResult<u64> {
        self.call::<_, _>(
            RpcRequest::GetMinimumBalanceForRentExemption,
            json!([data_len, config]),
            Some(64),
        )
        .await?
        .into_rpc_result()
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
    ) -> RpcResult<Vec<RpcKeyedAccount>> {
        self.call::<_, Vec<RpcKeyedAccount>>(
            RpcRequest::GetProgramAccounts,
            json!([program_id.to_string(), config]),
            self.config.response_size_estimate,
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Requests an airdrop of lamports to a Pubkey
    ///
    /// Method relies on the `requestAirdrop` RPC call to request the airdrop:
    ///   https://solana.com/docs/rpc/http/requestAirdrop
    ///
    pub async fn request_airdrop(&self, pubkey: &Pubkey, lamports: u64) -> RpcResult<String> {
        self.call::<_, String>(
            RpcRequest::RequestAirdrop,
            json!([pubkey.to_string(), lamports]),
            Some(156),
        )
        .await?
        .into_rpc_result()
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
        let default_limit = 1000;

        self.call::<_, Vec<RpcConfirmedTransactionStatusWithSignature>>(
            RpcRequest::GetSignaturesForAddress,
            json!([pubkey.to_string(), config]),
            Some(SIGNATURE_RESPONSE_SIZE_ESTIMATE * config.limit.unwrap_or(default_limit) as u64),
        )
        .await?
        .into_rpc_result()
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
        config: Option<RpcSignatureStatusConfig>,
    ) -> RpcResult<Vec<Option<TransactionStatus>>> {
        let signatures = signatures.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        if signatures.len() > 256 {
            return Err(RpcError::ValidationError(
                "Exceeded maximum signature limit of 256".to_string(),
            ));
        }

        // Estimate 256 bytes per transaction status to account for errors and metadata
        let max_response_bytes = signatures.len() as u64 * TRANSACTION_STATUS_RESPONSE_SIZE_ESTIMATE;

        self.call::<_, OptionalContext<Vec<Option<TransactionStatus>>>>(
            RpcRequest::GetSignatureStatuses,
            json!([signatures, config]),
            Some(max_response_bytes),
        )
        .await?
        .into_rpc_result()
        .map(|c| c.parse_value())
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
        config: Option<RpcTransactionConfig>,
    ) -> RpcResult<EncodedConfirmedTransactionWithStatusMeta> {
        self.call::<_, EncodedConfirmedTransactionWithStatusMeta>(
            RpcRequest::GetTransaction,
            json!([signature, config]),
            self.config
                .response_size_estimate
                .or(Some(TRANSACTION_RESPONSE_SIZE_ESTIMATE)),
        )
        .await?
        .into_rpc_result()
    }

    ///
    /// Returns the current Transaction count from the ledger.
    ///
    /// Method relies on the `getTransactionCount` RPC call to get the transaction count:
    ///   https://solana.com/docs/rpc/http/getTransactionCount
    ///
    pub async fn get_transaction_count(&self, config: Option<RpcContextConfig>) -> RpcResult<u64> {
        self.call::<_, u64>(RpcRequest::GetTransactionCount, json!([config]), Some(128))
            .await?
            .into_rpc_result()
    }

    ///
    /// Method relies on the `getTransaction` RPC call to get the transaction data:
    /// https://solana.com/docs/rpc/http/gettransaction
    /// It is using a batch request to get multiple transactions at once.
    ///
    /// cURL Example:
    /// curl -X POST -H "Content-Type: application/json" -d '[
    ///    {"jsonrpc":"2.0","id":1,"method":"getTransaction","params":["1"]}
    ///    {"jsonrpc":"2.0","id":2,"method":"getTransaction","params":["2"]}
    /// ]' http://localhost:8899
    ///
    pub async fn get_transactions(
        &self,
        signatures: Vec<&str>,
        config: Option<RpcTransactionConfig>,
    ) -> RpcResult<HashMap<String, RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>>> {
        let requests = signatures
            .iter()
            .map(|signature| (RpcRequest::GetTransaction, json!([signature, config])))
            .collect::<Vec<_>>();

        let response = self
            .batch_call::<_, EncodedConfirmedTransactionWithStatusMeta>(
                &requests,
                self.config
                    .response_size_estimate
                    .or_else(|| Some(signatures.len() as u64 * TRANSACTION_RESPONSE_SIZE_ESTIMATE)),
            )
            .await?;

        let result = response
            .into_iter()
            .enumerate()
            .map(|(index, res)| {
                let entity = if let Some(error) = res.error {
                    Err(RpcError::JsonRpcError(error))
                } else {
                    Ok(res.result)
                };
                (signatures[index].to_string(), entity)
            })
            .collect::<HashMap<_, _>>();

        Ok(result)
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
    pub async fn send_transaction(&self, tx: Transaction, config: RpcSendTransactionConfig) -> RpcResult<Signature> {
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

        let response = self
            .call::<_, String>(RpcRequest::SendTransaction, json!([raw_tx, config]), Some(156))
            .await?
            .into_rpc_result()?;

        Signature::from_str(&response).map_err(|_| RpcError::ParseError("Failed to parse signature".to_string()))
    }

    ///
    /// Calculate the max response bytes for the provided block range.
    ///
    fn get_block_range_max_response_bytes(start_slot: u64, limit: u64) -> u64 {
        let end_slot = start_slot.saturating_add(limit);
        let max_slot_str_len = end_slot.to_string().len() as u64;
        let commas_size = if limit > 0 { limit - 1 } else { 0 };
        36 + (max_slot_str_len * limit) + commas_size
    }

    ///
    /// Processes the result of an RPC method call by handling consistent and inconsistent responses from multiple providers.
    ///
    fn process_result<T: Serialize>(method: impl ToString, result: Result<T, MultiCallError<T>>) -> RpcResult<T> {
        match result {
            Ok(value) => Ok(value),
            Err(MultiCallError::ConsistentError(err)) => Err(err),
            Err(MultiCallError::InconsistentResults(multi_call_results)) => {
                let results = multi_call_results
                    .into_vec()
                    .into_iter()
                    .map(|(provider, result)| {
                        let cluster = provider.cluster();
                        add_metric_entry!(
                            inconsistent_responses,
                            (
                                MetricRpcMethod(method.to_string()),
                                MetricRpcHost(cluster.host_str().unwrap_or_else(|| "(unknown)".to_string()))
                            ),
                            1
                        );
                        Ok((provider, serde_json::to_string(&result?)?))
                    })
                    .collect::<Result<Vec<(RpcApi, String)>, RpcError>>()?;

                Err(RpcError::InconsistentResponse(results))
            }
        }
    }

    fn find_rpc_method_name(payload: &Value) -> &str {
        payload
            .pointer("/method")
            .or_else(|| payload.pointer("/0/method"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    }
}

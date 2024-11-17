use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    fmt::Debug,
    str::FromStr,
};

use base64::{prelude::BASE64_STANDARD, Engine};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use crate::{
    add_metric_entry,
    constants::*,
    request::RpcRequest,
    rpc_client::multi_call::{MultiCallError, MultiCallResults},
    types::{
        CommitmentConfig, EncodedConfirmedTransactionWithStatusMeta, Epoch, EpochInfo, EpochSchedule, Pubkey,
        RpcAccountInfoConfig, RpcBlockConfig, RpcBlockProductionConfig, RpcContextConfig, RpcEpochConfig,
        RpcGetVoteAccountsConfig, RpcLargestAccountsConfig, RpcLeaderScheduleConfig, RpcProgramAccountsConfig,
        RpcSendTransactionConfig, RpcSignatureStatusConfig, RpcSignaturesForAddressConfig,
        RpcSimulateTransactionConfig, RpcSupplyConfig, RpcTokenAccountsFilter, RpcTransactionConfig, Signature, Slot,
        Transaction, TransactionStatus, UiAccount, UiConfirmedBlock, UiTokenAmount, UiTransactionEncoding,
        UnixTimestamp,
    },
};

mod compression;
mod multi_call;
mod types;

pub use types::*;

use crate::{
    logs::DEBUG,
    metrics::{MetricRpcHost, MetricRpcMethod},
    rpc_client::compression::decompress_if_needed,
    types::{
        response::{
            OptionalContext, RpcAccountBalance, RpcBlockCommitment, RpcBlockProduction, RpcBlockhash,
            RpcConfirmedTransactionStatusWithSignature, RpcContactInfo, RpcIdentity, RpcInflationGovernor,
            RpcInflationRate, RpcInflationReward, RpcKeyedAccount, RpcLeaderSchedule, RpcPerfSample,
            RpcPrioritizationFee, RpcResponse, RpcSimulateTransactionResult, RpcSnapshotSlotInfo, RpcSupply,
            RpcVersionInfo, RpcVoteAccountStatus,
        },
        tagged::RpcTokenAccountBalance,
    },
};

thread_local! {
    static NEXT_ID: RefCell<u64> = RefCell::default();
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct RpcClientConfig {
    pub response_consensus: Option<ConsensusStrategy>,
    pub response_size_estimate: Option<u64>,
    pub request_cost_calculator: Option<RequestCostCalculator>,
    pub host_validator: Option<HostValidator>,
    pub transform_context: Option<TransformContext>,
    pub use_compression: bool,
    pub is_demo_active: bool,
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

    fn response_size_estimate(&self, estimate: u64) -> u64 {
        self.config
            .response_size_estimate
            .unwrap_or(estimate + HEADER_SIZE_LIMIT)
    }

    fn consensus_strategy(&self) -> ConsensusStrategy {
        self.config.response_consensus.as_ref().copied().unwrap_or_default()
    }

    /// Generate the next request id.
    pub fn next_request_id(&self) -> u64 {
        NEXT_ID.with(|next_id| {
            let mut next_id = next_id.borrow_mut();
            let id = *next_id;
            *next_id = next_id.wrapping_add(1);
            id
        })
    }

    /// Asynchronously sends an HTTP POST request to the specified URL with the given payload and
    /// maximum response bytes and returns the response as a string.
    /// This function calculates the required cycles for the HTTP request and logs the request
    /// details and response status. It uses a transformation named "cleanup_response" for the
    /// response body.
    ///
    /// # Arguments
    ///
    /// * `provider` - RPC API provider.
    /// * `payload` - JSON payload to be sent in the HTTP request.
    /// * `max_response_bytes` - The maximal size of the response in bytes. If None, 2MiB will be
    ///   the limit.
    ///
    /// # Returns
    ///
    /// * `RpcResult<Vec<u8>>` - The response body as a vector of bytes.
    async fn call_internal(
        &self,
        provider: &RpcApi,
        payload: &Value,
        max_response_bytes: Option<u64>,
    ) -> RpcResult<Vec<u8>> {
        let cluster = provider.cluster();
        let url = cluster.url();

        // Ensure "Content-Type: application/json" is present
        let mut headers = provider.headers.clone().unwrap_or_default();
        if !headers
            .iter()
            .any(|header| header.name.eq_ignore_ascii_case("Content-Type"))
        {
            headers.push(HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            });
        }

        if self.config.use_compression {
            headers.push(HttpHeader {
                name: "Accept-Encoding".to_string(),
                value: "gzip, deflate".to_string(),
            });
        }

        let body = serde_json::to_vec(payload).map_err(|e| RpcError::ParseError(e.to_string()))?;

        let request = CanisterHttpRequestArgument {
            url: url.to_string(),
            max_response_bytes,
            method: HttpMethod::POST,
            headers,
            body: Some(body),
            transform: self.config.transform_context.clone(),
        };

        // Calculate cycles if a calculator is provided
        let (cycles_cost, cycles_cost_with_collateral) = self
            .config
            .request_cost_calculator
            .as_ref()
            .map_or((0, 0), |calc| calc(&request));

        let parsed_url = url::Url::parse(url).map_err(|_| RpcError::ParseError(format!("Invalid URL: {}", url)))?;

        let host = parsed_url
            .host_str()
            .ok_or_else(|| RpcError::ParseError(format!("Error parsing hostname from URL: {}", url)))?;

        let rpc_host = MetricRpcHost(host.to_string());
        let rpc_method = MetricRpcMethod(Self::find_rpc_method_name(payload).to_string());

        if let Some(is_allowed) = self.config.host_validator {
            if !is_allowed(host) {
                add_metric_entry!(err_host_not_allowed, rpc_host.clone(), 1);
                return Err(RpcError::Text(format!("Disallowed RPC service host: {}", host)));
            }
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
                let bytes = if self.config.use_compression {
                    decompress_if_needed(response.body)?
                } else {
                    response.body
                };
                let body = std::str::from_utf8(&bytes).map_err(|e| RpcError::ParseError(e.to_string()))?;

                log!(
                    DEBUG,
                    "Got response (with {} bytes): {} from url: {} with status: {}",
                    body.len(),
                    body,
                    url,
                    response.status
                );

                // JSON-RPC responses over HTTP should have a 2xx status code,
                // even if the contained JsonRpcResult is an error.
                // If the server is not available, it will sometimes (wrongly) return HTML that will
                // fail to parse as JSON.
                let http_status: u16 = response.status.0.try_into().expect("Invalid http status code");
                // TODO: investigate
                // if !is_successful_http_code(&status) {
                //     return Err(RpcError::JsonRpcError { status, body }.into());
                // }

                add_metric_entry!(responses, (rpc_method, rpc_host, http_status.into()), 1);

                Ok(bytes)
            }
            Err(error) => {
                add_metric_entry!(err_http_outcall, (rpc_method, rpc_host), 1);
                Err(error.into())
            }
        }
    }

    /// Calls multiple providers in parallel and returns the results.
    async fn parallel_call(&self, payload: &Value, max_response_bytes: Option<u64>) -> Vec<RpcResult<Vec<u8>>> {
        futures::future::join_all(self.providers.iter().map(|provider| {
            log!(DEBUG, "[parallel_call]: will call provider: {:?}", provider);
            async { self.call_internal(provider, payload, max_response_bytes).await }
        }))
        .await
    }

    /// Makes a single JSON-RPC call.
    pub async fn call<P: Serialize, R: DeserializeOwned>(
        &self,
        method: RpcRequest,
        params: P,
        max_response_bytes: Option<u64>,
    ) -> RpcResult<JsonRpcResponse<R>> {
        let payload = method.build_json(self.next_request_id(), params);
        let results = self
            .parallel_call(
                &payload,
                max_response_bytes.map(|estimate| self.response_size_estimate(estimate)),
            )
            .await;
        let bytes = Self::process_result(
            method,
            MultiCallResults::from_non_empty_iter(self.providers.iter().cloned().zip(results.into_iter()))
                .reduce(self.consensus_strategy()),
        )?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Makes multiple JSON-RPC calls in a single batch request.
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

        let results = self
            .parallel_call(
                &payload,
                max_response_bytes.map(|estimate| self.response_size_estimate(estimate)),
            )
            .await;

        let bytes = Self::process_result(
            Self::find_rpc_method_name(&payload),
            MultiCallResults::from_non_empty_iter(self.providers.iter().cloned().zip(results.into_iter()))
                .reduce(self.consensus_strategy()),
        )?;

        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Returns all information associated with the account of the provided Pubkey.
    ///
    /// Method relies on the `getAccountInfo` RPC call to get the account info:
    ///   https://solana.com/docs/rpc/http/getAccountInfo
    pub async fn get_account_info(
        &self,
        pubkey: &Pubkey,
        config: Option<RpcAccountInfoConfig>,
    ) -> RpcResult<RpcResponse<Option<UiAccount>>> {
        self.call(
            RpcRequest::GetAccountInfo,
            (pubkey.to_string(), config),
            Some(MAX_PDA_ACCOUNT_DATA_LENGTH),
        )
        .await?
        .into()
    }

    /// Returns the lamport balance of the account of provided Pubkey.
    ///
    /// Method relies on the `getBalance` RPC call to get the balance:
    ///   https://solana.com/docs/rpc/http/getBalance
    pub async fn get_balance(
        &self,
        pubkey: &Pubkey,
        config: Option<RpcContextConfig>,
    ) -> RpcResult<OptionalContext<u64>> {
        self.call(RpcRequest::GetBalance, (pubkey.to_string(), config), Some(156))
            .await?
            .into()
    }

    /// Returns identity and transaction information about a confirmed block in the ledger.
    ///
    /// Method relies on the `getBlock` RPC call to get the block:
    ///   https://solana.com/docs/rpc/http/getBlock
    pub async fn get_block(&self, slot: Slot, config: Option<RpcBlockConfig>) -> RpcResult<UiConfirmedBlock> {
        self.call(
            RpcRequest::GetBlock,
            (slot, config.unwrap_or_default()),
            Some(GET_BLOCK_SIZE_ESTIMATE),
        )
        .await?
        .into()
    }

    /// Returns commitment for a particular block.
    ///
    /// Method relies on the `getBlockCommitment` RPC call to get the block commitment:
    ///   https://solana.com/docs/rpc/http/getBlockCommitment
    pub async fn get_block_commitment(&self, slot: Slot) -> RpcResult<RpcBlockCommitment> {
        self.call(
            RpcRequest::GetBlockCommitment,
            (slot,),
            Some(GET_BLOCK_COMMITMENT_SIZE_ESTIMATE),
        )
        .await?
        .into()
    }

    /// Returns the current block height of the node.
    ///
    /// Method relies on the `getBlockHeight` RPC call to get the block height:
    ///   https://solana.com/docs/rpc/http/getBlockHeight
    pub async fn get_block_height(&self, config: Option<RpcContextConfig>) -> RpcResult<u64> {
        self.call(RpcRequest::GetBlockHeight, (config.unwrap_or_default(),), Some(45))
            .await?
            .into()
    }

    /// Returns recent block production information from the current or previous epoch.
    ///
    /// Method relies on the `getBlockProduction` RPC call to get the block production:
    ///   https://solana.com/docs/rpc/http/getBlockProduction
    pub async fn get_block_production(
        &self,
        config: Option<RpcBlockProductionConfig>,
    ) -> RpcResult<OptionalContext<RpcBlockProduction>> {
        self.call(
            RpcRequest::GetBlockProduction,
            (config,),
            Some(GET_BLOCK_PRODUCTION_SIZE_ESTIMATE),
        )
        .await?
        .into()
    }

    /// Returns the estimated production time of a block.
    ///
    /// Method relies on the `getBlockTime` RPC call to get the block time:
    ///   https://solana.com/docs/rpc/http/getBlockTime
    pub async fn get_block_time(&self, slot: Slot) -> RpcResult<UnixTimestamp> {
        self.call(RpcRequest::GetBlockTime, (slot,), Some(GET_BLOCK_TIME_SIZE_ESTIMATE))
            .await?
            .into()
    }

    /// Returns a list of confirmed blocks between two slots.
    ///
    /// Method relies on the `getBlocks` RPC call to get the blocks:
    ///   https://solana.com/docs/rpc/http/getBlocks
    pub async fn get_blocks(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<Vec<u64>> {
        let params: Value = if end_slot.is_some() {
            json!([start_slot, end_slot, commitment_config])
        } else {
            json!([start_slot, commitment_config])
        };

        let end_slot = end_slot.unwrap_or(start_slot + MAX_GET_BLOCKS_RANGE);
        let limit = end_slot.saturating_sub(start_slot);

        if limit > MAX_GET_BLOCKS_RANGE {
            return Err(RpcError::ValidationError(format!(
                "Slot range too large; must be less or equal than {}",
                MAX_GET_BLOCKS_RANGE
            )));
        }

        self.call(
            RpcRequest::GetBlocks,
            params,
            Some(Self::get_block_range_max_response_bytes(start_slot, limit)),
        )
        .await?
        .into()
    }

    /// Returns a list of confirmed blocks starting at the given slot.
    ///
    /// Method relies on the `getBlocksWithLimit` RPC call to get the blocks with limit:
    ///   https://solana.com/docs/rpc/http/getBlocksWithLimit
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

        self.call(
            RpcRequest::GetBlocksWithLimit,
            (start_slot, limit, commitment_config),
            Some(Self::get_block_range_max_response_bytes(start_slot, limit)),
        )
        .await?
        .into()
    }

    /// Returns information about all the nodes participating in the cluster
    ///
    /// Method relies on the `getClusterNodes` RPC call to get the cluster nodes:
    ///   https://solana.com/docs/rpc/http/getClusterNodes
    pub async fn get_cluster_nodes(&self) -> RpcResult<Vec<RpcContactInfo>> {
        self.call(RpcRequest::GetClusterNodes, (), None).await?.into()
    }

    /// Returns information about the current epoch.
    ///
    /// Method relies on the `getEpochInfo` RPC call to get the epoch info:
    ///   https://solana.com/docs/rpc/http/getEpochInfo
    pub async fn get_epoch_info(&self, config: Option<RpcContextConfig>) -> RpcResult<EpochInfo> {
        self.call(RpcRequest::GetEpochInfo, (config,), Some(GET_EPOCH_INFO_SIZE_ESTIMATE))
            .await?
            .into()
    }

    /// Returns the epoch schedules information from this cluster's genesis config.
    ///
    /// Method relies on the `getEpochSchedule` RPC call to get the epoch schedule:
    ///   https://solana.com/docs/rpc/http/getEpochSchedule
    pub async fn get_epoch_schedule(&self) -> RpcResult<EpochSchedule> {
        self.call(RpcRequest::GetEpochSchedule, (), Some(GET_EPOCH_SCHEDULE_SIZE_ESTIMATE))
            .await?
            .into()
    }

    /// Get the fee the network will charge for a particular Message.
    ///
    /// Method relies on the `getFeeForMessage` RPC call to get the fee for a message:
    ///   https://solana.com/docs/rpc/http/getFeeForMessage
    pub async fn get_fee_for_message(
        &self,
        message: String,
        config: Option<RpcContextConfig>,
    ) -> RpcResult<OptionalContext<u64>> {
        self.call(RpcRequest::GetFeeForMessage, (message, config), Some(128))
            .await?
            .into()
    }

    /// Returns the slot of the lowest confirmed block that has not been purged from the ledger.
    ///
    /// Method relies on the `getFirstAvailableBlock` RPC call to get the first available block:
    ///   https://solana.com/docs/rpc/http/getFirstAvailableBlock
    pub async fn get_first_available_block(&self) -> RpcResult<u64> {
        self.call::<_, u64>(RpcRequest::GetFirstAvailableBlock, (), Some(128))
            .await?
            .into()
    }

    /// Returns the genesis hash.
    ///
    /// Method relies on the `getGenesisHash` RPC call to get the genesis hash:
    ///   https://solana.com/docs/rpc/http/getGenesisHash
    pub async fn get_genesis_hash(&self) -> RpcResult<String> {
        self.call(RpcRequest::GetGenesisHash, (), Some(128)).await?.into()
    }

    /// Returns the current health of the node.
    /// A healthy node is one that is within HEALTH_CHECK_SLOT_DISTANCE slots of the latest
    /// cluster-confirmed slot.
    ///
    /// Method relies on the `getHealth` RPC call to get the health status:
    ///   https://solana.com/docs/rpc/http/getHealth
    pub async fn get_health(&self) -> RpcResult<String> {
        self.call(RpcRequest::GetHealth, (), Some(128)).await?.into()
    }

    /// Returns the highest slot information that the node has snapshots for.
    /// This will find the highest full snapshot slot and the highest incremental
    /// snapshot slot based on the full snapshot slot, if there is one.
    ///
    /// Method relies on the `getHighestSnapshotSlot` RPC call to get the highest snapshot slot:
    ///   https://solana.com/docs/rpc/http/getHighestSnapshotSlot
    pub async fn get_highest_snapshot_slot(&self) -> RpcResult<RpcSnapshotSlotInfo> {
        self.call(RpcRequest::GetHighestSnapshotSlot, (), Some(256))
            .await?
            .into()
    }

    /// Returns the identity pubkey for the current node.
    ///
    /// Method relies on the `getIdentity` RPC call to get the identity pubkey:
    ///   https://solana.com/docs/rpc/http/getIdentity
    pub async fn get_identity(&self) -> RpcResult<RpcIdentity> {
        self.call(RpcRequest::GetIdentity, (), Some(128)).await?.into()
    }

    /// Returns the current inflation governor.
    ///
    /// Method relies on the `getInflationGovernor` RPC call to get inflation governor:
    ///   https://solana.com/docs/rpc/http/getInflationGovernor
    pub async fn get_inflation_governor(&self) -> RpcResult<RpcInflationGovernor> {
        self.call(RpcRequest::GetInflationGovernor, (), Some(256)).await?.into()
    }

    /// Returns the specific inflation values for the current epoch.
    ///
    /// Method relies on the `getInflationRate` RPC call to get inflation rate:
    ///   https://solana.com/docs/rpc/http/getInflationRate
    pub async fn get_inflation_rate(&self) -> RpcResult<RpcInflationRate> {
        self.call(RpcRequest::GetInflationRate, (), Some(256)).await?.into()
    }

    /// Returns the inflation / staking reward for a list of addresses for an epoch.
    ///
    /// Method relies on the `getInflationReward` RPC call to get inflation reward:
    ///   https://solana.com/docs/rpc/http/getInflationReward
    pub async fn get_inflation_reward(
        &self,
        addresses: &[Pubkey],
        config: Option<RpcEpochConfig>,
    ) -> RpcResult<Vec<Option<RpcInflationReward>>> {
        self.call(
            RpcRequest::GetInflationReward,
            (addresses, config),
            Some(40 + 153 * addresses.len() as u64),
        )
        .await?
        .into()
    }

    /// Returns the 20 largest accounts, by lamport balance (results may be cached up to two hours).
    ///
    /// Method relies on the `getLatestAccounts` RPC call to get the largest accounts:
    ///   https://solana.com/docs/rpc/http/getLatestAccounts
    pub async fn get_largest_accounts(
        &self,
        config: Option<RpcLargestAccountsConfig>,
    ) -> RpcResult<OptionalContext<Vec<RpcAccountBalance>>> {
        self.call(RpcRequest::GetLargestAccounts, (config,), Some(128 * 20))
            .await?
            .into()
    }

    /// Returns the latest blockhash.
    ///
    /// Method relies on the `getLatestBlockhash` RPC call to get the latest blockhash:
    ///   https://solana.com/docs/rpc/http/getLatestBlockhash
    pub async fn get_latest_blockhash(
        &self,
        config: Option<RpcContextConfig>,
    ) -> RpcResult<OptionalContext<RpcBlockhash>> {
        self.call(RpcRequest::GetLatestBlockhash, (config,), Some(156))
            .await?
            .into()
    }

    /// Returns the leader schedule for an epoch.
    ///
    /// Method relies on the `getLeaderSchedule` RPC call to get the leader schedule:
    ///   https://solana.com/docs/rpc/http/getLeaderSchedule
    pub async fn get_leader_schedule(
        &self,
        epoch: Epoch,
        config: Option<RpcLeaderScheduleConfig>,
    ) -> RpcResult<RpcLeaderSchedule> {
        self.call(RpcRequest::GetLeaderSchedule, (epoch, config), None)
            .await?
            .into()
    }

    /// Get the max slot seen from the retransmit stage.
    ///
    /// Method relies on the `getMaxRetransmitSlot` RPC call to get the max slot:
    ///   https://solana.com/docs/rpc/http/getMaxRetransmitSlot
    pub async fn get_max_retransmit_slot(&self) -> RpcResult<u64> {
        self.call(RpcRequest::GetMaxRetransmitSlot, (), None).await?.into()
    }

    /// Get the max slot seen from after shred insert.
    ///
    /// Method relies on the `getMaxShredInsertSlot` RPC call to get the max slot:
    ///   https://solana.com/docs/rpc/http/getMaxShredInsertSlot
    pub async fn get_max_shred_insert_slot(&self) -> RpcResult<u64> {
        self.call(RpcRequest::GetMaxShredInsertSlot, (), None).await?.into()
    }

    /// Returns a list of recent performance samples, in reverse slot order.
    /// Performance samples are taken every 60 seconds and include the number
    /// of transactions and slots that occur in a given time window.
    ///
    /// Method relies on the `getRecentPerformanceSamples` RPC call to get the performance samples:
    ///   https://solana.com/docs/rpc/http/getRecentPerformanceSamples
    pub async fn get_recent_performance_samples(&self, limit: u64) -> RpcResult<Vec<RpcPerfSample>> {
        self.call(RpcRequest::GetRecentPerformanceSamples, (limit,), Some(256 * limit))
            .await?
            .into()
    }

    /// Returns a list of prioritization fees from recent blocks.
    ///
    /// Method relies on the `getRecentPrioritizationFees` RPC call to get the prioritization fees:
    ///   https://solana.com/docs/rpc/http/getRecentPrioritizationFees
    pub async fn get_recent_prioritization_fees(&self, addresses: &[Pubkey]) -> RpcResult<Vec<RpcPrioritizationFee>> {
        self.call(
            RpcRequest::GetRecentPrioritizationFees,
            (addresses,),
            Some(128 * addresses.len() as u64),
        )
        .await?
        .into()
    }

    /// Returns the token balance of an SPL Token account.
    ///
    /// Method relies on the `getTokenAccountBalance` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenAccountBalance
    pub async fn get_token_account_balance(
        &self,
        pubkey: &Pubkey,
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<OptionalContext<UiTokenAmount>> {
        self.call(
            RpcRequest::GetTokenAccountBalance,
            (pubkey.to_string(), commitment_config),
            Some(256),
        )
        .await?
        .into()
    }

    /// Returns all SPL Token accounts by approved Delegate.
    ///
    /// Method relies on the `getTokenAccountsByDelegate` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenAccountsByDelegate
    pub async fn get_token_accounts_by_delegate(
        &self,
        pubkey: &Pubkey,
        filter: RpcTokenAccountsFilter,
        config: Option<RpcAccountInfoConfig>,
    ) -> RpcResult<OptionalContext<Vec<RpcKeyedAccount>>> {
        self.call(
            RpcRequest::GetTokenAccountsByDelegate,
            (pubkey.to_string(), filter, config),
            Some(GET_TOKEN_ACCOUNTS_SIZE_ESTIMATE),
        )
        .await?
        .into()
    }

    /// Returns all SPL Token accounts by token owner.
    ///
    /// Method relies on the `getTokenAccountsByOwner` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenAccountsByOwner
    pub async fn get_token_accounts_by_owner(
        &self,
        pubkey: &Pubkey,
        filter: RpcTokenAccountsFilter,
        config: Option<RpcAccountInfoConfig>,
    ) -> RpcResult<OptionalContext<Vec<RpcKeyedAccount>>> {
        self.call(
            RpcRequest::GetTokenAccountsByOwner,
            (pubkey.to_string(), filter, config),
            Some(GET_TOKEN_ACCOUNTS_SIZE_ESTIMATE),
        )
        .await?
        .into()
    }

    /// Returns the 20 largest accounts of a particular SPL Token type.
    ///
    /// Method relies on the `getTokenLargestAccounts` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenLargestAccounts
    pub async fn get_token_largest_accounts(
        &self,
        mint: &Pubkey,
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<OptionalContext<Vec<RpcTokenAccountBalance>>> {
        self.call(
            RpcRequest::GetTokenLargestAccounts,
            (mint.to_string(), commitment_config),
            Some(GET_TOKEN_LARGEST_ACCOUNTS_SIZE_ESTIMATE),
        )
        .await?
        .into()
    }

    /// Returns the total supply of an SPL Token type.
    ///
    /// Method relies on the `getTokenSupply` RPC call to get the token balance:
    ///   https://solana.com/docs/rpc/http/getTokenSupply
    pub async fn get_token_supply(
        &self,
        mint: &Pubkey,
        commitment_config: Option<CommitmentConfig>,
    ) -> RpcResult<OptionalContext<UiTokenAmount>> {
        self.call(
            RpcRequest::GetTokenSupply,
            (mint.to_string(), commitment_config),
            Some(GET_TOKEN_SUPPLY_SIZE_ESTIMATE),
        )
        .await?
        .into()
    }

    /// Returns the current Solana version running on the node.
    ///
    /// Method relies on the `getVersion` RPC call to get the version info:
    ///   https://solana.com/docs/rpc/http/getVersion
    pub async fn get_version(&self) -> RpcResult<RpcVersionInfo> {
        self.call(RpcRequest::GetVersion, (), Some(128)).await?.into()
    }

    /// Returns the account info and associated stake for all the voting accounts in the current
    /// bank.
    ///
    /// Method relies on the `getVoteAccounts` RPC call to get the voting accounts:
    ///   https://solana.com/docs/rpc/http/getVoteAccounts
    pub async fn get_vote_accounts(&self, config: Option<RpcGetVoteAccountsConfig>) -> RpcResult<RpcVoteAccountStatus> {
        self.call(
            RpcRequest::GetVoteAccounts,
            (config,),
            Some(GET_VOTE_ACCOUNTS_SIZE_ESTIMATE),
        )
        .await?
        .into()
    }

    /// Returns whether a blockhash is still valid or not.
    ///
    /// Method relies on the `isBlockhashValid` RPC call to check if the blockhash is valid:
    ///   https://solana.com/docs/rpc/http/isBlockhashValid
    pub async fn is_blockhash_valid(
        &self,
        blockhash: String,
        config: Option<RpcContextConfig>,
    ) -> RpcResult<OptionalContext<bool>> {
        self.call(RpcRequest::IsBlockhashValid, (blockhash, config), Some(128))
            .await?
            .into()
    }

    /// Returns the slot that has reached the given or default commitment level.
    ///
    /// Method relies on the `getSlot` RPC call to get the slot:
    ///   https://solana.com/docs/rpc/http/getSlot
    pub async fn get_slot(&self, config: Option<RpcContextConfig>) -> RpcResult<Slot> {
        self.call(RpcRequest::GetSlot, (config,), Some(128)).await?.into()
    }

    /// Returns the current slot leader.
    ///
    /// Method relies on the `getSlotLeader` RPC call to get the slot leader:
    ///   https://solana.com/docs/rpc/http/getSlotLeader
    pub async fn get_slot_leader(&self, config: Option<RpcContextConfig>) -> RpcResult<String> {
        self.call(RpcRequest::GetSlotLeader, (config,), Some(128)).await?.into()
    }

    /// Returns the slot leaders for a given slot range.
    ///
    /// Method relies on the `getSlotLeaders` RPC call to get the slot leaders:
    ///   https://solana.com/docs/rpc/http/getSlotLeaders
    pub async fn get_slot_leaders(&self, start_slot: u64, limit: Option<u64>) -> RpcResult<Vec<String>> {
        let limit = limit.unwrap_or(MAX_GET_SLOT_LEADERS);

        if limit > MAX_GET_SLOT_LEADERS {
            return Err(RpcError::ValidationError(format!(
                "Exceeded maximum limit of {}",
                MAX_GET_SLOT_LEADERS
            )));
        }

        let commas_size = if limit > 0 { limit - 1 } else { 0 };
        let max_response_bytes = 36 + 46 * limit + commas_size;

        self.call(
            RpcRequest::GetSlotLeaders,
            (start_slot, limit),
            Some(max_response_bytes),
        )
        .await?
        .into()
    }

    /// Returns the stake minimum delegation, in lamports.
    ///
    /// Method relies on the `getStakeMinimumDelegation` RPC call to get the supply:
    ///   https://solana.com/docs/rpc/http/getStakeMinimumDelegation
    pub async fn get_stake_minimum_delegation(&self, config: Option<CommitmentConfig>) -> RpcResult<RpcResponse<u64>> {
        self.call(RpcRequest::GetStakeMinimumDelegation, (config,), Some(128))
            .await?
            .into()
    }

    /// Returns information about the current supply.
    ///
    /// Method relies on the `getSupply` RPC call to get the supply:
    ///   https://solana.com/docs/rpc/http/getSupply
    pub async fn get_supply(&self, config: Option<RpcSupplyConfig>) -> RpcResult<RpcResponse<RpcSupply>> {
        self.call(RpcRequest::GetSupply, (config,), Some(GET_SUPPLY_SIZE_ESTIMATE))
            .await?
            .into()
    }

    /// Returns the minimum balance required to make account rent exempt.
    ///
    /// Method relies on the `getMinimumBalanceForRentExemption` RPC call to get the minimum balance
    /// for rent exemption: https://solana.com/docs/rpc/http/getMinimumBalanceForRentExemption
    pub async fn get_minimum_balance_for_rent_exemption(
        &self,
        data_len: usize,
        config: Option<CommitmentConfig>,
    ) -> RpcResult<u64> {
        self.call(
            RpcRequest::GetMinimumBalanceForRentExemption,
            (data_len, config),
            Some(64),
        )
        .await?
        .into()
    }

    /// Returns the account information for a list of Pubkeys.
    ///
    /// Method relies on the `getMultipleAccounts` RPC call to get multiple accounts:
    ///   https://solana.com/docs/rpc/http/getMultipleAccounts
    pub async fn get_multiple_accounts(
        &self,
        pubkeys: Vec<Pubkey>,
        config: Option<RpcAccountInfoConfig>,
    ) -> RpcResult<OptionalContext<Vec<UiAccount>>> {
        self.call(
            RpcRequest::GetMultipleAccounts,
            (&pubkeys, config),
            Some(pubkeys.len() as u64 * MAX_PDA_ACCOUNT_DATA_LENGTH),
        )
        .await?
        .into()
    }

    /// Returns all accounts owned by the provided program Pubkey.
    ///
    /// Method relies on the `getProgramAccounts` RPC call to get the program accounts:
    ///   https://solana.com/docs/rpc/http/getProgramAccounts
    pub async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        config: Option<RpcProgramAccountsConfig>,
    ) -> RpcResult<Vec<RpcKeyedAccount>> {
        self.call(
            RpcRequest::GetProgramAccounts,
            (program_id.to_string(), config.unwrap_or_default()),
            Some(100 * MAX_PDA_ACCOUNT_DATA_LENGTH),
        )
        .await?
        .into()
    }

    /// Returns the lowest slot that the node has information about in its ledger.
    ///
    /// Method relies on the `minimumLedgerSlot` RPC call to get the minimum ledger slot:
    ///   https://solana.com/docs/rpc/http/minimumLedgerSlot
    pub async fn minimum_ledger_slot(&self) -> RpcResult<Slot> {
        self.call(RpcRequest::MinimumLedgerSlot, (), Some(64)).await?.into()
    }

    /// Requests an airdrop of lamports to a Pubkey
    ///
    /// Method relies on the `requestAirdrop` RPC call to request the airdrop:
    ///   https://solana.com/docs/rpc/http/requestAirdrop
    pub async fn request_airdrop(&self, pubkey: &Pubkey, lamports: u64) -> RpcResult<String> {
        self.call(RpcRequest::RequestAirdrop, (pubkey.to_string(), lamports), Some(156))
            .await?
            .into()
    }

    /// Returns signatures for confirmed transactions that include the given address in their
    /// accountKeys list.
    /// Returns signatures backwards in time from the provided signature or
    /// the most recent confirmed block.
    ///
    /// Method relies on the `getSignaturesForAddress` RPC call to get the signatures for the
    /// address: https://solana.com/docs/rpc/http/getsignaturesforaddress
    pub async fn get_signatures_for_address(
        &self,
        pubkey: &Pubkey,
        config: Option<RpcSignaturesForAddressConfig>,
    ) -> RpcResult<Vec<RpcConfirmedTransactionStatusWithSignature>> {
        let limit = config.as_ref().and_then(|c| c.limit).unwrap_or(1000) as u64;
        self.call(
            RpcRequest::GetSignaturesForAddress,
            (pubkey.to_string(), config),
            Some(SIGNATURE_RESPONSE_SIZE_ESTIMATE * limit),
        )
        .await?
        .into()
    }

    /// Returns the statuses of a list of transaction signatures.
    ///
    /// Method relies on the `getSignatureStatuses` RPC call to get the statuses for the signatures:
    ///   https://solana.com/docs/rpc/http/getSignatureStatuses
    pub async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
        config: Option<RpcSignatureStatusConfig>,
    ) -> RpcResult<OptionalContext<Vec<Option<TransactionStatus>>>> {
        let signatures = signatures.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        if signatures.len() > 256 {
            return Err(RpcError::ValidationError(
                "Exceeded maximum signature limit of 256".to_string(),
            ));
        }

        let max_response_bytes = signatures.len() as u64 * TRANSACTION_STATUS_RESPONSE_SIZE_ESTIMATE;

        self.call(
            RpcRequest::GetSignatureStatuses,
            (signatures, config),
            Some(max_response_bytes),
        )
        .await?
        .into()
    }

    /// Returns transaction details for a confirmed transaction.
    ///
    /// Method relies on the `getTransaction` RPC call to get the transaction data:
    ///   https://solana.com/docs/rpc/http/getTransaction
    pub async fn get_transaction(
        &self,
        signature: &Signature,
        config: Option<RpcTransactionConfig>,
    ) -> RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>> {
        self.call(
            RpcRequest::GetTransaction,
            (signature.to_string(), config.unwrap_or_default()),
            Some(TRANSACTION_RESPONSE_SIZE_ESTIMATE),
        )
        .await?
        .into_optional_rpc_result()
    }

    /// Returns the current transactions count from the ledger.
    ///
    /// Method relies on the `getTransactionCount` RPC call to get the transaction count:
    ///   https://solana.com/docs/rpc/http/getTransactionCount
    pub async fn get_transaction_count(&self, config: Option<RpcContextConfig>) -> RpcResult<u64> {
        self.call(RpcRequest::GetTransactionCount, (config,), Some(128))
            .await?
            .into()
    }

    /// Method relies on the `getTransaction` RPC call to get the transaction data:
    /// https://solana.com/docs/rpc/http/gettransaction
    /// It is using a batch request to get multiple transactions at once.
    ///
    /// cURL Example:
    /// curl -X POST -H "Content-Type: application/json" -d '[
    ///    {"jsonrpc":"2.0","id":1,"method":"getTransaction","params":["1"]}
    ///    {"jsonrpc":"2.0","id":2,"method":"getTransaction","params":["2"]}
    /// ]' http://localhost:8899
    pub async fn get_transactions(
        &self,
        signatures: Vec<&str>,
        config: Option<RpcTransactionConfig>,
    ) -> RpcResult<HashMap<String, RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>>> {
        let requests = signatures
            .iter()
            .map(|signature| (RpcRequest::GetTransaction, (signature, config.unwrap_or_default())))
            .collect::<Vec<_>>();

        let response = self
            .batch_call::<_, EncodedConfirmedTransactionWithStatusMeta>(
                &requests,
                Some(signatures.len() as u64 * TRANSACTION_RESPONSE_SIZE_ESTIMATE),
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

    /// Submits a signed transaction to the cluster for processing.
    /// This method does not alter the transaction in any way; it relays the transaction created by
    /// clients to the node as-is.
    /// If the node's rpc service receives the transaction, this
    /// method immediately succeeds, without waiting for any confirmations.
    /// A successful response from this method does not guarantee the transaction is processed or
    /// confirmed by the cluster.
    ///
    /// Use [RpcClient::get_signature_statuses] to ensure a transaction is processed and confirmed.
    ///
    /// Method relies on the `sendTransaction` RPC call to send the transaction:
    ///   https://solana.com/docs/rpc/http/sendTransaction
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

        let response: RpcResult<String> = self
            .call(RpcRequest::SendTransaction, (raw_tx, config), Some(156))
            .await?
            .into();

        Signature::from_str(response?.as_str())
            .map_err(|_| RpcError::ParseError("Failed to parse signature".to_string()))
    }

    /// Simulates sending a transaction.
    ///
    /// Method relies on the `simulateTransaction` RPC call to simulate the transaction:
    ///   https://solana.com/docs/rpc/http/simulateTransaction
    pub async fn simulate_transaction(
        &self,
        tx: Transaction,
        config: RpcSimulateTransactionConfig,
    ) -> RpcResult<OptionalContext<RpcSimulateTransactionResult>> {
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

        self.call(RpcRequest::SimulateTransaction, (raw_tx, config), None)
            .await?
            .into()
    }

    /// Processes the result of an RPC method call by handling consistent and inconsistent responses
    /// from multiple providers.
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

    /// Calculate the max response bytes for the provided block range.
    fn get_block_range_max_response_bytes(start_slot: u64, limit: u64) -> u64 {
        let end_slot = start_slot.saturating_add(limit);
        let max_slot_str_len = end_slot.to_string().len() as u64;
        let commas_size = if limit > 0 { limit - 1 } else { 0 };
        36 + (max_slot_str_len * limit) + commas_size
    }

    /// Extracts the JSON-RPC `method` name from the request payload.
    ///
    /// This function searches for the `method` field within the provided JSON-RPC
    /// request payload. It handles both single and batch requests:
    ///
    /// - **Single Request**: Retrieves the `method` directly from the payload.
    /// - **Batch Request**: Retrieves the `method` from the first request in the batch.
    ///
    /// If the `method` field is not found, in either case returns `"unknown"`.
    fn find_rpc_method_name(payload: &Value) -> &str {
        payload
            .pointer("/method")
            .or_else(|| payload.pointer("/0/method"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    }
}

// TODO:
// pub fn is_response_too_large(code: &RejectionCode, message: &str) -> bool {
//     code == &RejectionCode::SysFatal && (message.contains("size limit") ||
// message.contains("length limit")) }
//
// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub struct ResponseSizeEstimate(u64);
//
// impl ResponseSizeEstimate {
//     pub fn new(num_bytes: u64) -> Self {
//         assert!(num_bytes > 0);
//         assert!(num_bytes <= MAX_PAYLOAD_SIZE);
//         Self(num_bytes)
//     }
//
//     /// Describes the expected (90th percentile) number of bytes in the HTTP response body.
//     /// This number should be lower than `MAX_PAYLOAD_SIZE`.
//     pub fn get(self) -> u64 {
//         self.0
//     }
//
//     /// Returns a higher estimate for the payload size.
//     pub fn adjust(self) -> Self {
//         Self(self.0.max(1024).saturating_mul(2).min(MAX_PAYLOAD_SIZE))
//     }
// }
//
// impl std::fmt::Display for ResponseSizeEstimate {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }

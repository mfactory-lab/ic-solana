use {
    crate::{
        constants::{
            CANISTER_OVERHEAD, COLLATERAL_CYCLES_PER_NODE, HTTP_OUTCALL_REQUEST_BASE_COST,
            HTTP_OUTCALL_REQUEST_COST_PER_BYTE, HTTP_OUTCALL_REQUEST_PER_NODE_COST,
            HTTP_OUTCALL_RESPONSE_COST_PER_BYTE, INGRESS_MESSAGE_BYTE_RECEIVED_COST, INGRESS_MESSAGE_RECEIVED_COST,
            INGRESS_OVERHEAD_BYTES, NODES_IN_SUBNET, RPC_HOSTS_BLOCKLIST, RPC_URL_COST_BYTES,
        },
        providers::find_provider,
        state::read_state,
        types::{RpcConfig, RpcServices},
    },
    ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder},
    ic_cdk::api::management_canister::http_request::TransformContext,
    ic_solana::rpc_client::{RpcClient, RpcClientConfig},
};

///
/// Create an [RpcClient] based on the provided configuration.
///
pub fn rpc_client(source: RpcServices, config: Option<RpcConfig>) -> RpcClient {
    let providers = match source {
        RpcServices::Provider(providers) => providers
            .iter()
            .map(|pid| {
                let provider =
                    find_provider(pid).unwrap_or_else(|| ic_cdk::trap(&format!("Provider {} not found", pid)));
                provider.api()
            })
            .collect(),
        RpcServices::Custom(apis) => apis,
    };

    let config = config.unwrap_or_default();

    read_state(|s| {
        let config = RpcClientConfig {
            response_consensus: config.response_consensus,
            response_size_estimate: config.response_size_estimate,
            cost_calculator: Some(|s| {
                let cycles_cost = get_http_request_cost(
                    s.body.as_ref().map_or(0, |b| b.len() as u64),
                    s.max_response_bytes.unwrap_or(2 * 1024 * 1024),
                );
                (cycles_cost, get_cost_with_collateral(cycles_cost))
            }),
            transform_context: Some(TransformContext::from_name("__transform_json_rpc".to_owned(), vec![])),
            is_demo_active: s.is_demo_active,
            hosts_blocklist: RPC_HOSTS_BLOCKLIST,
            extra_response_bytes: 0,
        };

        RpcClient::new(providers, Some(config))
    })
}

/// Calculates the cost of sending a JSON-RPC request using HTTP outcalls.
pub fn get_http_request_cost(payload_size_bytes: u64, max_response_bytes: u64) -> u128 {
    let nodes_in_subnet = NODES_IN_SUBNET as u128;
    let ingress_bytes = payload_size_bytes as u128 + RPC_URL_COST_BYTES as u128 + INGRESS_OVERHEAD_BYTES;
    let cost_per_node = INGRESS_MESSAGE_RECEIVED_COST
        + INGRESS_MESSAGE_BYTE_RECEIVED_COST * ingress_bytes
        + HTTP_OUTCALL_REQUEST_BASE_COST
        + HTTP_OUTCALL_REQUEST_PER_NODE_COST * nodes_in_subnet
        + HTTP_OUTCALL_REQUEST_COST_PER_BYTE * payload_size_bytes as u128
        + HTTP_OUTCALL_RESPONSE_COST_PER_BYTE * max_response_bytes as u128
        + CANISTER_OVERHEAD;
    cost_per_node * nodes_in_subnet
}

/// Calculate the cost + collateral cycles for an HTTP request.
pub fn get_cost_with_collateral(cycles_cost: u128) -> u128 {
    cycles_cost + COLLATERAL_CYCLES_PER_NODE * NODES_IN_SUBNET as u128
}

/// Return an HttpResponse that lists this canister's metrics
pub fn serve_metrics(
    encode_metrics: impl FnOnce(&mut ic_metrics_encoder::MetricsEncoder<Vec<u8>>) -> std::io::Result<()>,
) -> HttpResponse {
    let mut writer = ic_metrics_encoder::MetricsEncoder::new(vec![], ic_cdk::api::time() as i64 / 1_000_000);

    match encode_metrics(&mut writer) {
        Ok(()) => HttpResponseBuilder::ok()
            .header("Content-Type", "text/plain; version=0.0.4")
            .with_body_and_content_length(writer.into_inner())
            .build(),
        Err(err) => HttpResponseBuilder::server_error(format!("Failed to encode metrics: {}", err)).build(),
    }
}

pub fn serve_logs(request: HttpRequest) -> HttpResponse {
    use {
        ic_solana_common::logs::{Log, Priority, Sort},
        std::str::FromStr,
    };

    let max_skip_timestamp = match request.raw_query_param("time") {
        Some(arg) => match u64::from_str(arg) {
            Ok(value) => value,
            Err(_) => {
                return HttpResponseBuilder::bad_request()
                    .with_body_and_content_length("failed to parse the 'time' parameter")
                    .build()
            }
        },
        None => 0,
    };

    let mut log: Log = Default::default();

    match request.raw_query_param("priority").map(Priority::from_str) {
        Some(Ok(priority)) => match priority {
            Priority::Info => log.push_logs(Priority::Info),
            Priority::Debug => log.push_logs(Priority::Debug),
        },
        _ => {
            log.push_logs(Priority::Info);
            log.push_logs(Priority::Debug);
        }
    }

    log.entries.retain(|entry| entry.timestamp >= max_skip_timestamp);

    fn ordering_from_query_params(sort: Option<&str>, max_skip_timestamp: u64) -> Sort {
        match sort {
            Some(ord_str) => match Sort::from_str(ord_str) {
                Ok(order) => order,
                Err(_) => {
                    if max_skip_timestamp == 0 {
                        Sort::Ascending
                    } else {
                        Sort::Descending
                    }
                }
            },
            None => {
                if max_skip_timestamp == 0 {
                    Sort::Ascending
                } else {
                    Sort::Descending
                }
            }
        }
    }

    log.sort_logs(ordering_from_query_params(
        request.raw_query_param("sort"),
        max_skip_timestamp,
    ));

    const MAX_BODY_SIZE: usize = 3_000_000;
    HttpResponseBuilder::ok()
        .header("Content-Type", "application/json; charset=utf-8")
        .with_body_and_content_length(log.serialize_logs(MAX_BODY_SIZE))
        .build()
}

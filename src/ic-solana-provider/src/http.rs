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
    },
    ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder},
    ic_cdk::api::management_canister::http_request::TransformContext,
    ic_solana::{
        rpc_client::{MultiCallError, RpcClient},
        types::result::MultiRpcResult,
    },
    ic_solana_common::{add_metric_entry, metrics::MetricRpcHost},
};

fn process_result<T>(method: RpcMethod, result: Result<T, MultiCallError<T>>) -> MultiRpcResult<T> {
    match result {
        Ok(value) => MultiRpcResult::Consistent(Ok(value)),
        Err(err) => match err {
            MultiCallError::ConsistentError(err) => MultiRpcResult::Consistent(Err(err)),
            MultiCallError::InconsistentResults(multi_call_results) => {
                let results = multi_call_results.into_vec();
                results.iter().for_each(|(service, _service_result)| {
                    if let Ok(ResolvedRpcService::Provider(provider)) = resolve_rpc_service(service.clone()) {
                        add_metric_entry!(
                            inconsistent_responses,
                            (
                                method.into(),
                                MetricRpcHost(provider.hostname().unwrap_or_else(|| "(unknown)".to_string()))
                            ),
                            1
                        )
                    }
                });
                MultiRpcResult::Inconsistent(results)
            }
        },
    }
}

pub fn rpc_client(provider_id: &str) -> RpcClient {
    let provider =
        find_provider(provider_id).unwrap_or_else(|| ic_cdk::trap(&format!("Provider {} not found", provider_id)));

    let api = provider.api();

    let client = read_state(|s| {
        RpcClient::new(&api.url)
            .with_demo(s.is_demo_active)
            .with_hosts_blocklist(RPC_HOSTS_BLOCKLIST)
            .with_request_cost_calculator(|s| {
                let cycles_cost = get_http_request_cost(
                    s.body.as_ref().map_or(0, |b| b.len() as u64),
                    s.max_response_bytes.unwrap_or(2 * 1024 * 1024),
                );
                (cycles_cost, get_cost_with_collateral(cycles_cost))
            })
            .with_transform_context(TransformContext::from_name("__transform_json_rpc".to_owned(), vec![]))
    });

    if let Some(headers) = api.headers {
        client.with_headers(headers)
    } else {
        client
    }
}

// pub fn rpc_client(provider_id: &str) -> RpcClient {
//     if let Some(provider) = find_provider(provider_id) {
//         let api = provider.api();
//
//         let client = read_state(|s| {
//             RpcClient::new(&api.url)
//                 .with_demo(s.is_demo_active)
//                 .with_hosts_blocklist(RPC_HOSTS_BLOCKLIST)
//                 .with_request_cost_calculator(|s| {
//                     let cycles_cost = get_http_request_cost(
//                         s.body.as_ref().map_or(0, |b| b.len() as u64),
//                         s.max_response_bytes.unwrap_or(2 * 1024 * 1024), // default 2Mb
//                     );
//                     (cycles_cost, get_cost_with_collateral(cycles_cost))
//                 })
//                 .with_transform_context(TransformContext::from_name("__transform_json_rpc".to_owned(), vec![]))
//         });
//
//         if let Some(headers) = api.headers {
//             client.with_headers(headers)
//         } else {
//             client
//         }
//     } else {
//         ic_cdk::trap(&format!("Provider {} not found", provider_id));
//     }
// }

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

use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_solana::{
    constants::HTTP_MAX_SIZE,
    logs::{Log, Priority, Sort},
    rpc_client::{RpcApi, RpcClient, RpcClientConfig, RpcConfig, RpcServices},
    types::Cluster,
};

use crate::{
    constants::{
        COLLATERAL_CYCLES_PER_NODE, DEFAULT_SUBNET_SIZE, HTTP_REQUEST_LINEAR_BASELINE_FEE, HTTP_REQUEST_PER_BYTE_FEE,
        HTTP_REQUEST_QUADRATIC_BASELINE_FEE, HTTP_RESPONSE_PER_BYTE_FEE, INGRESS_BYTE_RECEPTION_FEE,
        INGRESS_MESSAGE_RECEPTION_FEE, INGRESS_OVERHEAD_BYTES, NODES_IN_SUBNET, RPC_URL_COST_BYTES,
    },
    providers::find_provider,
    state::read_state,
    utils::validate_hostname,
};

/// Create an [RpcClient] based on the provided configuration.
pub fn rpc_client(source: RpcServices, config: Option<RpcConfig>) -> RpcClient {
    let providers = match source {
        RpcServices::Mainnet | RpcServices::Testnet | RpcServices::Devnet | RpcServices::Localnet => {
            let cluster = match source {
                RpcServices::Mainnet => Cluster::Mainnet,
                RpcServices::Testnet => Cluster::Testnet,
                RpcServices::Devnet => Cluster::Devnet,
                RpcServices::Localnet => Cluster::Localnet,
                _ => unreachable!(),
            };
            vec![get_provider_rpc_api(cluster.as_ref())]
        }
        RpcServices::Provider(ids) => ids.iter().map(|id| get_provider_rpc_api(id)).collect(),
        RpcServices::Custom(apis) => apis, // Use the custom APIs directly
    };

    let config = config.unwrap_or_default();

    read_state(|s| {
        let config = RpcClientConfig {
            response_consensus: config.response_consensus,
            response_size_estimate: config.response_size_estimate,
            request_cost_calculator: Some(|req| {
                let cycles_cost = get_http_request_cost(
                    req.body.as_ref().map_or(0, |bytes| bytes.len() as u64),
                    req.max_response_bytes.unwrap_or(HTTP_MAX_SIZE),
                );
                (cycles_cost, get_cost_with_collateral(cycles_cost))
            }),
            host_validator: Some(|host| validate_hostname(host).is_ok()),
            transform_function_name: Some("__transform_json_rpc".to_owned()),
            is_demo_active: s.is_demo_active,
            use_compression: false,
        };
        RpcClient::new(providers, Some(config))
    })
}

/// Retrieve the [RpcApi] from a provider ID.
fn get_provider_rpc_api(provider_id: &str) -> RpcApi {
    find_provider(provider_id)
        .unwrap_or_else(|| ic_cdk::trap(&format!("Unknown provider `{}`", provider_id)))
        .api()
}

/// Calculates the baseline cost of sending a request using HTTP outcalls.
/// The corresponding code in replica:
/// https://github.com/dfinity/ic/blob/master/rs/cycles_account_manager/src/lib.rs#L1153
pub fn get_http_request_cost(payload_size_bytes: u64, max_response_bytes: u64) -> u128 {
    let subnet_size = NODES_IN_SUBNET as u128;
    let request_size = payload_size_bytes as u128;
    let response_size = max_response_bytes as u128;
    let ingress_size = request_size + RPC_URL_COST_BYTES as u128 + INGRESS_OVERHEAD_BYTES;

    (INGRESS_MESSAGE_RECEPTION_FEE / DEFAULT_SUBNET_SIZE as u128
        + INGRESS_BYTE_RECEPTION_FEE / DEFAULT_SUBNET_SIZE as u128 * ingress_size
        + HTTP_REQUEST_LINEAR_BASELINE_FEE
        + HTTP_REQUEST_QUADRATIC_BASELINE_FEE * subnet_size
        + HTTP_REQUEST_PER_BYTE_FEE * request_size
        + HTTP_RESPONSE_PER_BYTE_FEE * response_size)
        * subnet_size
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
    use std::str::FromStr;

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_request_cost() {
        let payload = r#"{"jsonrpc":"2.0","method":"sol_getHealth","params":[],"id":1}"#;
        let base_cost = get_http_request_cost(payload.len() as u64, 1000);
        let base_cost_10_extra_bytes = get_http_request_cost(payload.len() as u64 + 10, 1000);
        let estimated_cost_10_extra_bytes = base_cost
            + 10 * (HTTP_REQUEST_PER_BYTE_FEE + INGRESS_BYTE_RECEPTION_FEE / DEFAULT_SUBNET_SIZE as u128)
                * NODES_IN_SUBNET as u128;
        assert_eq!(base_cost_10_extra_bytes, estimated_cost_10_extra_bytes);
    }

    #[test]
    fn test_candid_rpc_cost() {
        assert_eq!(
            [
                get_http_request_cost(0, 0),
                get_http_request_cost(123, 123),
                get_http_request_cost(123, 4567890),
                get_http_request_cost(890, 4567890),
            ],
            [176350350, 182008596, 124425270996, 124439692130]
        );
    }

    #[test]
    fn test1() {
        let payload = r#"{"jsonrpc":"2.0","method":"sol_getHealth","params":[],"id":1}"#;
        let cost = get_http_request_cost(payload.len() as u64, 128u64);
        println!("cost: {}", cost);
    }
}

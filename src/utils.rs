mod option_serializer;

pub use option_serializer::*;

use crate::constants::{
    CANISTER_OVERHEAD, HTTP_OUTCALL_REQUEST_BASE_COST, HTTP_OUTCALL_REQUEST_COST_PER_BYTE,
    HTTP_OUTCALL_REQUEST_PER_NODE_COST, HTTP_OUTCALL_RESPONSE_COST_PER_BYTE,
    INGRESS_MESSAGE_BYTE_RECEIVED_COST, INGRESS_MESSAGE_RECEIVED_COST, INGRESS_OVERHEAD_BYTES,
    RPC_URL_MIN_COST_BYTES,
};
use crate::state::read_state;
use candid::Principal;

pub fn debug_println_caller(method_name: &str) {
    ic_cdk::println!(
        "{}: caller: {} (isAnonymous: {})",
        method_name,
        ic_cdk::caller().to_text(),
        ic_cdk::caller() == Principal::anonymous()
    );
}

/// Calculates the baseline cost of sending a JSON-RPC request using HTTP outcalls.
pub fn get_http_request_cost(url: &str, payload_size_bytes: u64, max_response_bytes: u64) -> u128 {
    let nodes_in_subnet = read_state(|s| s.nodes_in_subnet);
    let ingress_bytes = payload_size_bytes as u128
        + u32::max(RPC_URL_MIN_COST_BYTES, url.len() as u32) as u128
        + INGRESS_OVERHEAD_BYTES;
    let cost_per_node = INGRESS_MESSAGE_RECEIVED_COST
        + INGRESS_MESSAGE_BYTE_RECEIVED_COST * ingress_bytes
        + HTTP_OUTCALL_REQUEST_BASE_COST
        + HTTP_OUTCALL_REQUEST_PER_NODE_COST * nodes_in_subnet as u128
        + HTTP_OUTCALL_REQUEST_COST_PER_BYTE * payload_size_bytes as u128
        + HTTP_OUTCALL_RESPONSE_COST_PER_BYTE * max_response_bytes as u128
        + CANISTER_OVERHEAD;
    cost_per_node * (nodes_in_subnet as u128)
}

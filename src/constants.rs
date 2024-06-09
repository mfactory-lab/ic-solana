// HTTP outcall cost calculation
// See https://internetcomputer.org/docs/current/developer-docs/gas-cost#special-features
pub const INGRESS_OVERHEAD_BYTES: u128 = 100;
pub const INGRESS_MESSAGE_RECEIVED_COST: u128 = 1_200_000;
pub const INGRESS_MESSAGE_BYTE_RECEIVED_COST: u128 = 2_000;
pub const HTTP_OUTCALL_REQUEST_BASE_COST: u128 = 3_000_000;
pub const HTTP_OUTCALL_REQUEST_PER_NODE_COST: u128 = 60_000;
pub const HTTP_OUTCALL_REQUEST_COST_PER_BYTE: u128 = 400;
pub const HTTP_OUTCALL_RESPONSE_COST_PER_BYTE: u128 = 800;

// Additional cost of operating the canister per subnet node
pub const CANISTER_OVERHEAD: u128 = 1_000_000;

// Minimum number of bytes charged for a URL; improves consistency of costs between providers
pub const RPC_URL_MIN_COST_BYTES: u32 = 256;

pub const NODES_IN_STANDARD_SUBNET: u32 = 13;
pub const NODES_IN_FIDUCIARY_SUBNET: u32 = 28;

pub const PDA_ACCOUNT_MAX_SIZE: u64 = 10 * 1024;

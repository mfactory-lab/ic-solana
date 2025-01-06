/// HTTP outcall cost calculation
/// See https://internetcomputer.org/docs/current/developer-docs/gas-cost#cycles-price-breakdown
/// and https://github.com/dfinity/ic/blob/b9c732eace54b47292969e77801e22317ae182a2/rs/config/src/subnet_config.rs#L442
pub const INGRESS_OVERHEAD_BYTES: u128 = 100;
pub const INGRESS_MESSAGE_RECEPTION_FEE: u128 = 1_200_000;
pub const INGRESS_BYTE_RECEPTION_FEE: u128 = 2_000;
pub const HTTP_REQUEST_LINEAR_BASELINE_FEE: u128 = 3_000_000;
pub const HTTP_REQUEST_QUADRATIC_BASELINE_FEE: u128 = 60_000;
pub const HTTP_REQUEST_PER_BYTE_FEE: u128 = 400;
pub const HTTP_RESPONSE_PER_BYTE_FEE: u128 = 800;

/// Additional cost of operating the canister per subnet node
pub const CANISTER_OVERHEAD: u128 = 1_000_000;

/// Cycles which must be passed with each RPC request in case the
/// third-party JSON-RPC prices increase in the future (currently always refunded)
pub const COLLATERAL_CYCLES_PER_NODE: u128 = 10_000_000;

/// Minimum number of bytes charged for a URL; improves consistency of costs between providers
pub const RPC_URL_COST_BYTES: u32 = 256;

/// Default subnet size which is used to scale cycles cost according to a subnet replication factor.
pub const DEFAULT_SUBNET_SIZE: u32 = 13;

pub const NODES_IN_SUBNET: u32 = 34;

pub const PROVIDER_ID_MAX_SIZE: u32 = 128;

/// List of hosts which are not allowed to be used as RPC providers
pub const RPC_HOSTS_BLOCKLIST: &[&str] = &[];

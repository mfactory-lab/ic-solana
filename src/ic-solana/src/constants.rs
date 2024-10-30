// This constant is our approximation of the expected header size.
// The HTTP standard doesn't define any limit, and many implementations limit
// the headers size to 8 KiB. We chose a lower limit because headers observed on most providers
// fit in the constant defined below, and if there is a spike, then the payload size adjustment
// should take care of that.
pub const HEADER_SIZE_LIMIT: u64 = 2 * 1024;

// This constant comes from the IC specification:
// See https://internetcomputer.org/docs/current/developer-docs/smart-contracts/maintain/resource-limits
pub const HTTP_MAX_SIZE: u64 = 2 * 1024 * 1024;

pub const MAX_PAYLOAD_SIZE: u64 = HTTP_MAX_SIZE - HEADER_SIZE_LIMIT;

// /// Maximum permitted size of account data (10 MiB).
// pub const MAX_ACCOUNT_DATA_LENGTH: u64 = 10 * 1024 * 1024;

/// Maximum permitted size of PDA account data (10 KiB).
/// However, a PDA can be resized up to the 10 MB limit.
pub const MAX_PDA_ACCOUNT_DATA_LENGTH: u64 = 10 * 1024;

/// In case no memo is set signature object should be around 175 bytes long.
pub const SIGNATURE_RESPONSE_SIZE_ESTIMATE: u64 = 500;

/// In case no memo is set, a transaction object should be around 1100 bytes long.
pub const TRANSACTION_RESPONSE_SIZE_ESTIMATE: u64 = 5 * 1024;

pub const TRANSACTION_STATUS_RESPONSE_SIZE_ESTIMATE: u64 = 256;

pub const GET_BLOCK_SIZE_ESTIMATE: u64 = 1024 * 1024; // 1 MiB
pub const GET_BLOCK_COMMITMENT_SIZE_ESTIMATE: u64 = 1024;
pub const GET_BLOCK_PRODUCTION_SIZE_ESTIMATE: u64 = 100_000;
pub const GET_BLOCK_TIME_SIZE_ESTIMATE: u64 = 128;
pub const GET_SUPPLY_SIZE_ESTIMATE: u64 = 1024;
pub const GET_TOKEN_SUPPLY_SIZE_ESTIMATE: u64 = 512;
pub const GET_EPOCH_INFO_SIZE_ESTIMATE: u64 = 128;
pub const GET_EPOCH_SCHEDULE_SIZE_ESTIMATE: u64 = 128;
pub const GET_TOKEN_ACCOUNTS_SIZE_ESTIMATE: u64 = 1400;
pub const GET_TOKEN_LARGEST_ACCOUNTS_SIZE_ESTIMATE: u64 = 256 * 20;
pub const GET_VOTE_ACCOUNTS_SIZE_ESTIMATE: u64 = 10000;

pub const MAX_GET_BLOCKS_RANGE: u64 = 500_000;
pub const MAX_GET_SLOT_LEADERS: u64 = 5000;

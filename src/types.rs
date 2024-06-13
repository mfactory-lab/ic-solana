pub mod account;
pub mod blockhash;
mod compiled_keys;
pub mod config;
pub mod epoch_info;
pub mod fee_calculator;
pub mod filter;
pub mod instruction;
pub mod instruction_error;
pub mod message;
pub mod reward;
pub mod signature;
pub mod transaction;
pub mod transaction_error;

/// The unit of time a given leader schedule is honored.
///
/// It lasts for some number of [`Slot`]s.
pub type Epoch = u64;

/// The unit of time given to a leader for encoding a block.
///
/// It is some number of _ticks_ long.
pub type Slot = u64;

/// An approximate measure of real-world time.
///
/// Expressed as Unix time (i.e. seconds since the Unix epoch).
pub type UnixTimestamp = i64;

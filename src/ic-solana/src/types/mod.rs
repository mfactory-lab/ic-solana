pub mod account;
pub mod block;
pub mod blockhash;
pub mod candid_value;
pub mod cluster;
pub mod commitment;
pub mod compiled_keys;
pub mod config;
pub mod epoch;
pub mod fee_calculator;
pub mod filter;
pub mod instruction;
pub mod message;
pub mod pubkey;
pub mod reward;
pub mod signature;
pub mod tagged;
pub mod transaction;
pub mod transaction_error;

pub use {
    account::*, block::*, blockhash::*, candid_value::*, cluster::*, commitment::*, config::*, epoch::*,
    fee_calculator::*, filter::*, instruction::*, message::*, pubkey::*, reward::*, signature::*, tagged::*,
    transaction::*, transaction_error::*,
};

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

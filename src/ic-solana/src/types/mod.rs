pub mod account;
pub mod block;
pub mod blockhash;
pub mod candid_value;
pub mod cluster;
pub mod commitment;
pub mod compiled_keys;
pub mod config;
pub mod epoch;
pub mod fees;
pub mod filter;
pub mod instruction;
pub mod message;
pub mod pubkey;
pub mod response;
pub mod reward;
pub mod signature;
pub mod tagged;
pub mod transaction;
pub mod transaction_error;

pub use account::*;
pub use block::*;
pub use blockhash::*;
pub use candid_value::*;
pub use cluster::*;
pub use commitment::*;
pub use config::*;
pub use epoch::*;
pub use fees::*;
pub use filter::*;
pub use instruction::*;
pub use message::*;
pub use pubkey::*;
pub use response::*;
pub use reward::*;
pub use signature::*;
pub use transaction::*;
pub use transaction_error::*;

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

use std::{fmt, mem, str::FromStr};

use serde::{Deserialize, Serialize};

/// Size of a hash in bytes.
pub const HASH_BYTES: usize = 32;

/// Maximum string length of a base58 encoded hash.
const MAX_BASE58_LEN: usize = 44;

#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct BlockHash(pub [u8; HASH_BYTES]);

impl Default for BlockHash {
    fn default() -> Self {
        Self([0u8; HASH_BYTES])
    }
}

impl BlockHash {
    pub fn new(hash_slice: &[u8]) -> Self {
        Self(<[u8; HASH_BYTES]>::try_from(hash_slice).unwrap())
    }
}

impl AsRef<[u8]> for BlockHash {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl fmt::Debug for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseHashError {
    #[error("string decoded to wrong size for hash")]
    WrongSize,
    #[error("failed to decoded string to hash")]
    Invalid,
}

impl FromStr for BlockHash {
    type Err = ParseHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > MAX_BASE58_LEN {
            return Err(ParseHashError::WrongSize);
        }
        let bytes = bs58::decode(s).into_vec().map_err(|_| ParseHashError::Invalid)?;
        if bytes.len() != mem::size_of::<BlockHash>() {
            Err(ParseHashError::WrongSize)
        } else {
            Ok(BlockHash::new(&bytes))
        }
    }
}

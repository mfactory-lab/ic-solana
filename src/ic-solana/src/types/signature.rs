use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// Number of bytes in a signature
pub const SIGNATURE_BYTES: usize = 64;

/// Maximum string length of a base58 encoded signature
const MAX_BASE58_SIGNATURE_LEN: usize = 88;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "BigArray")] pub(crate) [u8; SIGNATURE_BYTES]);

impl Default for Signature {
    fn default() -> Self {
        Self([0u8; SIGNATURE_BYTES])
    }
}

impl Signature {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or_default()
    }

    // pub(self) fn verify_verbose(
    //     &self,
    //     pubkey_bytes: &[u8],
    //     message_bytes: &[u8],
    // ) -> Result<(), ed25519_dalek::SignatureError> {
    //     let publickey = ed25519_dalek::PublicKey::from_bytes(pubkey_bytes)?;
    //     let signature = self.0.as_slice().try_into()?;
    //     publickey.verify_strict(message_bytes, &signature)
    // }
    //
    // pub fn verify(&self, pubkey_bytes: &[u8], message_bytes: &[u8]) -> bool {
    //     self.verify_verbose(pubkey_bytes, message_bytes).is_ok()
    // }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}

impl From<Signature> for [u8; SIGNATURE_BYTES] {
    fn from(signature: Signature) -> Self {
        signature.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseSignatureError {
    #[error("string decoded to wrong size for signature")]
    WrongSize,
    #[error("failed to decode string to signature")]
    Invalid,
}

impl FromStr for Signature {
    type Err = ParseSignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > MAX_BASE58_SIGNATURE_LEN {
            return Err(ParseSignatureError::WrongSize);
        }
        let bytes = bs58::decode(s).into_vec().map_err(|_| ParseSignatureError::Invalid)?;
        Signature::try_from(bytes).map_err(|_| ParseSignatureError::WrongSize)
    }
}

impl<'a> TryFrom<&'a [u8]> for Signature {
    type Error = <[u8; SIGNATURE_BYTES] as TryFrom<&'a [u8]>>::Error;

    #[inline]
    fn try_from(signature: &'a [u8]) -> Result<Self, Self::Error> {
        <[u8; SIGNATURE_BYTES]>::try_from(signature).map(Self)
    }
}

impl TryFrom<Vec<u8>> for Signature {
    type Error = <[u8; SIGNATURE_BYTES] as TryFrom<Vec<u8>>>::Error;

    #[inline]
    fn try_from(signature: Vec<u8>) -> Result<Self, Self::Error> {
        <[u8; SIGNATURE_BYTES]>::try_from(signature).map(Self)
    }
}

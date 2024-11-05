use std::{
    fmt::{self, Display},
    str::FromStr,
};

use candid::{CandidType, Principal};
use ic_management_canister_types::{
    DerivationPath, SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs, SchnorrPublicKeyResponse,
    SignWithSchnorrArgs, SignWithSchnorrReply,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

// https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
pub const EDDSA_SIGN_COST: u128 = 26_153_846_153;

#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub enum SchnorrKey {
    TestKeyLocal,
    TestKey1,
    ProductionKey1,
    Custom(String),
}

impl Display for SchnorrKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-schnorr
        let key_str = match self {
            SchnorrKey::TestKeyLocal => "dfx_test_key1",
            SchnorrKey::TestKey1 => "test_key_1",
            SchnorrKey::ProductionKey1 => "key_1",
            SchnorrKey::Custom(key) => key,
        };
        f.write_str(key_str)
    }
}

impl FromStr for SchnorrKey {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dfx_test_key1" => Ok(SchnorrKey::TestKeyLocal),
            "test_key_1" => Ok(SchnorrKey::TestKey1),
            "key_1" => Ok(SchnorrKey::ProductionKey1),
            _ => Ok(SchnorrKey::Custom(s.to_string())),
        }
    }
}

/// Fetches the ed25519 public key from the schnorr canister.
pub async fn eddsa_public_key(key: SchnorrKey, derivation_path: Vec<ByteBuf>) -> Vec<u8> {
    let res: Result<(SchnorrPublicKeyResponse,), _> = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: DerivationPath::new(derivation_path),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: key.to_string(),
            },
        },),
    )
    .await;

    res.expect("Failed to fetch ed25519 public key").0.public_key
}

/// Signs a message with an ed25519 key.
pub async fn sign_with_eddsa(key: SchnorrKey, derivation_path: Vec<ByteBuf>, message: Vec<u8>) -> Vec<u8> {
    ic_cdk::api::call::msg_cycles_accept128(EDDSA_SIGN_COST);

    let res: Result<(SignWithSchnorrReply,), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_schnorr",
        (SignWithSchnorrArgs {
            message,
            derivation_path: DerivationPath::new(derivation_path),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: key.to_string(),
            },
        },),
        EDDSA_SIGN_COST as u64,
    )
    .await;

    res.expect("Failed to sign with ed25519").0.signature
}

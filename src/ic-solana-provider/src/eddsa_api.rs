use {
    crate::state::read_state,
    candid::Principal,
    ic_management_canister_types::{
        DerivationPath, SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs,
        SchnorrPublicKeyResponse, SignWithSchnorrArgs, SignWithSchnorrReply,
    },
    serde_bytes::ByteBuf,
};

/// Fetches the ed25519 public key from the schnorr canister.
pub async fn eddsa_public_key(key_name: String, derivation_path: Vec<ByteBuf>) -> Vec<u8> {
    let canister_id = read_state(|s| Principal::from_text(&s.schnorr_canister).unwrap());

    let res: Result<(SchnorrPublicKeyResponse,), _> = ic_cdk::call(
        canister_id,
        "schnorr_public_key",
        (SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: DerivationPath::new(derivation_path),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: key_name,
            },
        },),
    )
    .await;

    res.unwrap().0.public_key
}

/// Signs a message with an ed25519 key.
pub async fn sign_with_eddsa(
    key_name: String,
    derivation_path: Vec<ByteBuf>,
    message: Vec<u8>,
) -> Vec<u8> {
    let canister_id = read_state(|s| Principal::from_text(&s.schnorr_canister).unwrap());

    let res: Result<(SignWithSchnorrReply,), _> = ic_cdk::call(
        canister_id,
        "sign_with_schnorr",
        (SignWithSchnorrArgs {
            message,
            derivation_path: DerivationPath::new(derivation_path),
            key_id: SchnorrKeyId {
                name: key_name,
                algorithm: SchnorrAlgorithm::Ed25519,
            },
        },),
    )
    .await;

    res.unwrap().0.signature
}

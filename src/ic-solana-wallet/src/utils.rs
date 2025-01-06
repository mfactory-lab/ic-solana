use candid::Principal;
use ic_solana::types::Pubkey;
use serde_bytes::ByteBuf;

use crate::{
    eddsa::{eddsa_public_key, sign_with_eddsa},
    state::read_state,
};

pub fn validate_caller_not_anonymous() -> Principal {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        panic!("Anonymous principal not allowed to make calls.")
    }
    caller
}

pub async fn caller_pubkey(caller: Principal) -> Pubkey {
    let key_name = read_state(|s| s.schnorr_key.to_owned());
    let derived_path = vec![ByteBuf::from(caller.as_slice())];
    let pk = eddsa_public_key(key_name, derived_path).await;
    Pubkey::try_from(pk.as_slice()).expect("Invalid public key")
}

pub async fn caller_sign(caller: Principal, message: &[u8]) -> Vec<u8> {
    let key_name = read_state(|s| s.schnorr_key.to_owned());
    let derived_path = vec![ByteBuf::from(caller.as_slice())];
    sign_with_eddsa(key_name, derived_path, message.into()).await
}

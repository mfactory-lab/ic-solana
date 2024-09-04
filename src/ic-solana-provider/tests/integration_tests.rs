mod common;

use {
    crate::common::init,
    candid::{encode_one, Principal},
    common::USER_PRINCIPAL,
    pocket_ic::{PocketIc, PocketIcBuilder},
};

#[ctor::ctor]
fn init_vars() {
    std::env::set_var("SCHNORR_CANISTER_PATH", "schnorr_canister.wasm.gz");
    std::env::set_var("IC_SOLANA_PROVIDER_PATH", "ic-solana-provider.wasm.gz");
}

// #[test]
// fn test_get_address() {
//     let ic = PocketIc::new();

//     let canister_id = init(&ic);

//     ic.update_call(
//         canister_id,
//         USER_PRINCIPAL.clone(),
//         "get_address",
//         encode_one(()).unwrap(),
//     )
//     .unwrap();
// }

// #[test]
// #[should_panic(expected = "Anonymous principal not allowed to make calls.")]
// fn test_get_address_anonymous() {
//     let ic = PocketIc::new();
//     let canister_id = init(&ic);

//     ic.update_call(
//         canister_id,
//         Principal::anonymous(),
//         "get_address",
//         encode_one(()).unwrap(),
//     )
//     .unwrap();
// }

#[test]
fn test_get_balance() {
    // let ic = PocketIc::new();
    let mut ic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .with_ii_subnet()
        .with_sns_subnet()
        .build();

    ic.auto_progress();
    ic.make_live(None);

    let canister_id = init(&ic);

    let account = "AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY";

    let call = ic
        .submit_call(
            canister_id,
            USER_PRINCIPAL.clone(),
            "sol_getBalance",
            encode_one(account).unwrap(),
        )
        .unwrap();

    println!("REQUESTS: {:#?}", ic.get_canister_http());

    let result = ic.await_call(call);

    println!("RESULT: {:#?}", result);
}

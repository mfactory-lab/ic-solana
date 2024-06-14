use std::cell::RefCell;

use candid::Principal;
use ic_cdk::{init, query};

type GetAddressResponse = Result<String, String>;

thread_local! {
    static SOL_PROVIDER_CANISTER: RefCell<Option<Principal>>  = const { RefCell::new(None) };
}

/// The whoami method returns the calling principal and the sol address of the caller. A prerequisite
/// for a successful response is that the test canister has been initialized with a reference to the siws
/// provider canister and that the caller has completed the authentication process with the siws
/// provider canister.
#[query]
async fn whoami() -> Result<(String, String), String> {
    let calling_principal = ic_cdk::caller();

    // Get the sol provider canister reference
    let sol_provider_canister = SOL_PROVIDER_CANISTER
        .with_borrow(|canister| canister.expect("Solana provider canister not initialized"));

    let response: Result<(GetAddressResponse,), _> = ic_cdk::call(
        sol_provider_canister,
        "get_address",
        (calling_principal.as_slice(),),
    )
    .await;

    let address = match response {
        Ok(inner_result) => {
            // Handle the inner Result (GetAddressResponse)
            match inner_result.0 {
                Ok(address) => address,  // Successfully got the address
                Err(e) => return Err(e), // Handle error in GetAddressResponse
            }
        }
        Err(_) => return Err("Failed to get the caller address".to_string()), // Handle ic_cdk::call error
    };

    // Return the calling principal and address
    Ok((calling_principal.to_text(), address))
}

/// When setting up the test canister, we need to save a reference to the solana provider canister
/// so that we can call it later.
#[init]
async fn init(c: String) {
    SOL_PROVIDER_CANISTER.with(|canister| {
        *canister.borrow_mut() = Some(Principal::from_text(c).expect("Invalid principal"));
    });
}

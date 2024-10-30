use candid::Principal;

pub fn validate_caller_not_anonymous() -> Principal {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        panic!("Anonymous principal not allowed to make calls.")
    }
    caller
}

use {crate::state::read_state, candid::Principal, ic_solana::rpc_client::RpcClient};

pub fn rpc_client() -> RpcClient {
    read_state(|s| RpcClient::new(&s.rpc_url).with_nodes_in_subnet(s.nodes_in_subnet))
}

pub fn validate_caller_not_anonymous() -> Principal {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        panic!("Anonymous principal not allowed to make calls.")
    }
    caller
}

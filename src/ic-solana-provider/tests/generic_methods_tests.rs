mod common;

use {
    candid::{encode_args, encode_one, Decode},
    common::{init_with_rpc_url, BASIC_IDENTITY, SECRET1, SECRET2, SOLANA_DEVNET_CLUSTER_URL},
    ic_agent::Agent,
    pocket_ic::PocketIcBuilder,
    solana_sdk::{bs58, signer::Signer},
};

#[tokio::test]
async fn test_get_address() {
    const EXPECTED_ADDRESS: &str = "6uCnfVzwsAGJAhRj3H3iWcyitTbYt8a7yDvGga2LHqJf";

    let mut pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .build_async()
        .await;

    let endpoint = pic.make_live(None).await;

    // Create an agent for the PocketIC instance.
    let agent = Agent::builder()
        .with_url(endpoint)
        .with_identity(BASIC_IDENTITY.clone())
        .build()
        .unwrap();
    agent.fetch_root_key().await.unwrap();

    let canister_id = init_with_rpc_url(&pic, SOLANA_DEVNET_CLUSTER_URL).await;

    let call_result = agent
        .update(&canister_id, "get_address")
        .with_arg(encode_one(()).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let address = Decode!(&call_result, String);

    assert_eq!(address.unwrap(), EXPECTED_ADDRESS);
}

#[tokio::test]
async fn test_request_cost() {
    // The amount to send from one account to the other, in lamports.
    const AMOUNT_TO_SEND: u64 = 1;
    const MAX_RESPONSE_BYTES: u64 = 1000;
    const EXPECTED_COST: u128 = 159264000;

    let mut pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .build_async()
        .await;

    let endpoint = pic.make_live(None).await;

    // Create an agent for the PocketIC instance.
    let agent = Agent::builder().with_url(endpoint).build().unwrap();
    agent.fetch_root_key().await.unwrap();

    let canister_id = init_with_rpc_url(&pic, SOLANA_DEVNET_CLUSTER_URL).await;

    let solana_client = solana_client::nonblocking::rpc_client::RpcClient::new(
        SOLANA_DEVNET_CLUSTER_URL.to_string(),
    );

    let keypair1 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET1).unwrap();
    let keypair2 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET2).unwrap();
    let pubkey2 = keypair2.pubkey();

    // Getting latest blockhash
    let latest_blockhash = solana_client.get_latest_blockhash().await.unwrap();

    // Creating a transaction to send 2 SOL from keypair1 to keypair2
    let tx = solana_sdk::system_transaction::transfer(
        &keypair1,
        &pubkey2,
        AMOUNT_TO_SEND,
        latest_blockhash,
    );
    let serialized = bincode::serialize(&tx).unwrap();
    let raw_tx = bs58::encode(serialized).into_string();

    let signature = agent
        .update(&canister_id, "requestCost")
        .with_arg(encode_args((raw_tx, MAX_RESPONSE_BYTES)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let cost = Decode!(&signature, u128).unwrap();

    assert_eq!(cost, EXPECTED_COST);
}

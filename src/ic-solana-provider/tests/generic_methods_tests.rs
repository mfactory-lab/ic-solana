mod common;

use {
    candid::{encode_args, encode_one, Decode, Principal},
    common::{
        decode_raw_wasm_result, init, BASIC_IDENTITY, CONTROLLER_PRINCIPAL, SECRET1, SECRET2,
        SOLANA_DEVNET_CLUSTER_URL, SOLANA_MAINNET_CLUSTER_URL, SOLANA_TESTNET_CLUSTER_URL,
        USER_PRINCIPAL,
    },
    ic_agent::Agent,
    ic_solana_provider::{auth::Auth, types::RegisterProviderArgs},
    pocket_ic::PocketIcBuilder,
    solana_sdk::{bs58, signer::Signer},
};

#[tokio::test]
#[ignore = "This test is not working, as the Schnorr signature is not currently supported in pocket-ic"]
async fn test_sol_address() {
    const EXPECTED_ADDRESS: &str = "6uCnfVzwsAGJAhRj3H3iWcyitTbYt8a7yDvGga2LHqJf";

    let mut pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .with_ii_subnet()
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

    let canister_id = init(&pic).await;

    let call_result = agent
        .update(&canister_id, "sol_address")
        .with_arg(encode_one(()).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let address = Decode!(&call_result, String);

    assert_eq!(address.unwrap(), EXPECTED_ADDRESS);
}

// milti_thread is needed for solana_client to work
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_request_cost() {
    // The amount to send from one account to the other, in lamports.
    const AMOUNT_TO_SEND: u64 = 1;
    const MAX_RESPONSE_BYTES: u64 = 1000;
    const EXPECTED_COST: u128 = 254665600;

    let mut pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .build_async()
        .await;

    let endpoint = pic.make_live(None).await;

    // Create an agent for the PocketIC instance.
    let agent = Agent::builder().with_url(endpoint).build().unwrap();
    agent.fetch_root_key().await.unwrap();

    let canister_id = init(&pic).await;

    let solana_client =
        solana_client::rpc_client::RpcClient::new(SOLANA_DEVNET_CLUSTER_URL.to_string());

    let keypair1 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET1).unwrap();
    let keypair2 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET2).unwrap();
    let pubkey2 = keypair2.pubkey();

    // Getting latest blockhash
    let latest_blockhash = solana_client.get_latest_blockhash().unwrap();

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

#[tokio::test]
async fn test_providers() {
    let mut pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .build_async()
        .await;

    let _ = pic.make_live(None).await;

    let canister_id = init(&pic).await;

    // Authorize the controller to register one provider
    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "authorize",
        encode_args((CONTROLLER_PRINCIPAL.clone(), Auth::RegisterProvider)).unwrap(),
    )
    .await
    .unwrap();

    let register_mainnet_provider_args = RegisterProviderArgs {
        id: "test_mainnet".to_string(),
        url: SOLANA_MAINNET_CLUSTER_URL.to_string(),
        auth: None,
    };

    let register_devnet_provider_args = RegisterProviderArgs {
        id: "test_devnet".to_string(),
        url: SOLANA_DEVNET_CLUSTER_URL.to_string(),
        auth: None,
    };

    let register_testnet_provider_args = RegisterProviderArgs {
        id: "test_testnet".to_string(),
        url: SOLANA_TESTNET_CLUSTER_URL.to_string(),
        auth: None,
    };

    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "registerProvider",
        encode_one(register_mainnet_provider_args.clone()).unwrap(),
    )
    .await
    .unwrap();

    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "registerProvider",
        encode_one(register_devnet_provider_args.clone()).unwrap(),
    )
    .await
    .unwrap();

    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "registerProvider",
        encode_one(register_testnet_provider_args.clone()).unwrap(),
    )
    .await
    .unwrap();

    let result = pic
        .query_call(
            canister_id,
            CONTROLLER_PRINCIPAL.clone(),
            "getProviders",
            encode_one(()).unwrap(),
        )
        .await
        .unwrap();

    let (providers,): (Vec<String>,) = decode_raw_wasm_result(&result).unwrap();

    assert!(providers.contains(&"test_mainnet".to_string()));
    assert!(!providers.contains(&"test_devnet".to_string()));
    assert!(!providers.contains(&"test_testnet".to_string()));
    assert!(providers.contains(&"mainnet".to_string()));
    assert!(providers.contains(&"devnet".to_string()));
    assert!(providers.contains(&"testnet".to_string()));

    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "unregisterProvider",
        encode_one("test_mainnet".to_string()).unwrap(),
    )
    .await
    .unwrap();

    let result = pic
        .query_call(
            canister_id,
            CONTROLLER_PRINCIPAL.clone(),
            "getProviders",
            encode_one(()).unwrap(),
        )
        .await
        .unwrap();

    let (providers,): (Vec<String>,) = decode_raw_wasm_result(&result).unwrap();

    assert!(!providers.contains(&"test_mainnet".to_string()));
    assert!(!providers.contains(&"test_devnet".to_string()));
    assert!(!providers.contains(&"test_testnet".to_string()));
    assert!(providers.contains(&"mainnet".to_string()));
    assert!(providers.contains(&"devnet".to_string()));
    assert!(providers.contains(&"testnet".to_string()));

    // Authorize the controller to register one provider
    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "authorize",
        encode_args((CONTROLLER_PRINCIPAL.clone(), Auth::RegisterProvider)).unwrap(),
    )
    .await
    .unwrap();

    // Authorize the controller to register another one provider
    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "authorize",
        encode_args((CONTROLLER_PRINCIPAL.clone(), Auth::RegisterProvider)).unwrap(),
    )
    .await
    .unwrap();

    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "registerProvider",
        encode_one(register_devnet_provider_args.clone()).unwrap(),
    )
    .await
    .unwrap();

    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "registerProvider",
        encode_one(register_testnet_provider_args.clone()).unwrap(),
    )
    .await
    .unwrap();

    let result = pic
        .query_call(
            canister_id,
            CONTROLLER_PRINCIPAL.clone(),
            "getProviders",
            encode_one(()).unwrap(),
        )
        .await
        .unwrap();

    let (providers,): (Vec<String>,) = decode_raw_wasm_result(&result).unwrap();

    // Only one auth works
    assert!(!providers.contains(&"test_mainnet".to_string()));
    assert!(providers.contains(&"test_devnet".to_string()));
    assert!(!providers.contains(&"test_testnet".to_string()));
    assert!(providers.contains(&"mainnet".to_string()));
    assert!(providers.contains(&"devnet".to_string()));
    assert!(providers.contains(&"testnet".to_string()));
}

#[tokio::test]
async fn test_auth() {
    let mut pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .build_async()
        .await;

    let _ = pic.make_live(None).await;

    let canister_id = init(&pic).await;

    // Authorize the controller to register one provider
    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "authorize",
        encode_args((CONTROLLER_PRINCIPAL.clone(), Auth::RegisterProvider)).unwrap(),
    )
    .await
    .unwrap();

    let result = pic
        .query_call(
            canister_id,
            CONTROLLER_PRINCIPAL.clone(),
            "getAuthorized",
            encode_one(Auth::RegisterProvider).unwrap(),
        )
        .await
        .unwrap();

    let (auth,): (Vec<Principal>,) = decode_raw_wasm_result(&result).unwrap();

    assert_eq!(auth, vec![CONTROLLER_PRINCIPAL.clone()]);

    // Deauthorize the controller to register one provider
    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "deauthorize",
        encode_args((CONTROLLER_PRINCIPAL.clone(), Auth::RegisterProvider)).unwrap(),
    )
    .await
    .unwrap();

    let result = pic
        .query_call(
            canister_id,
            CONTROLLER_PRINCIPAL.clone(),
            "getAuthorized",
            encode_one(Auth::RegisterProvider).unwrap(),
        )
        .await
        .unwrap();

    let (auth,): (Vec<Principal>,) = decode_raw_wasm_result(&result).unwrap();

    assert!(auth.is_empty());

    // Trying to authorize from a non authorized controller
    pic.update_call(
        canister_id,
        USER_PRINCIPAL.clone(),
        "authorize",
        encode_args((USER_PRINCIPAL.clone(), Auth::Manage)).unwrap(),
    )
    .await
    .unwrap();

    let result = pic
        .query_call(
            canister_id,
            USER_PRINCIPAL.clone(),
            "getAuthorized",
            encode_one(Auth::Manage).unwrap(),
        )
        .await
        .unwrap();

    let (auth,): (Vec<Principal>,) = decode_raw_wasm_result(&result).unwrap();

    assert_eq!(auth, vec![CONTROLLER_PRINCIPAL.clone()]);

    // Trying to authorize two manage auth
    pic.update_call(
        canister_id,
        CONTROLLER_PRINCIPAL.clone(),
        "authorize",
        encode_args((CONTROLLER_PRINCIPAL.clone(), Auth::Manage)).unwrap(),
    )
    .await
    .unwrap();

    let result = pic
        .query_call(
            canister_id,
            USER_PRINCIPAL.clone(),
            "getAuthorized",
            encode_one(Auth::Manage).unwrap(),
        )
        .await
        .unwrap();

    let (auth,): (Vec<Principal>,) = decode_raw_wasm_result(&result).unwrap();

    assert_eq!(auth, vec![CONTROLLER_PRINCIPAL.clone()]);
}

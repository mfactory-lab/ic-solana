mod common;

use {
    crate::common::init,
    candid::{encode_args, encode_one, Decode},
    common::{
        BASIC_IDENTITY, DEVNET_PROVIDER_ID, MAINNET_PROVIDER_ID, SECRET1, SECRET2,
        SOLANA_DEVNET_CLUSTER_URL,
    },
    ic_agent::Agent,
    ic_solana::{
        rpc_client::RpcError,
        types::{Account, EncodedConfirmedTransactionWithStatusMeta, UiTokenAmount},
    },
    ic_solana_provider::types::SendTransactionRequest,
    pocket_ic::PocketIcBuilder,
    solana_sdk::{
        bs58, native_token::sol_to_lamports, pubkey::Pubkey, signature::Signature, signer::Signer,
        system_instruction,
    },
    std::str::FromStr,
};

#[tokio::test]
async fn test_get_balance() {
    pub const ACCOUNT: &str = "AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY";

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

    let res = agent
        .update(&canister_id, "sol_getBalance")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, ACCOUNT)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let _ = Decode!(&res, ic_solana::rpc_client::RpcResult<u64>).unwrap();
}

#[tokio::test]
async fn test_get_token_balance() {
    pub const TOKEN_ACCOUNT: &str = "2mMRrstGWsueujXeQnUUgp3VZBhJt8FWxroYDec6eCUw";

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

    let res = agent
        .update(&canister_id, "sol_getTokenBalance")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, TOKEN_ACCOUNT)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let _ = Decode!(&res, ic_solana::rpc_client::RpcResult<UiTokenAmount>)
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_get_latest_blockhash() {
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

    let res = agent
        .update(&canister_id, "sol_getLatestBlockhash")
        .with_arg(encode_args((MAINNET_PROVIDER_ID,)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let _ = Decode!(&res, ic_solana::rpc_client::RpcResult<String>)
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_get_account_info() {
    const EXISTING_ACCOUNT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    const NON_EXISTING_ACCOUNT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYd";

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

    let existing_account_res = agent
        .update(&canister_id, "sol_getAccountInfo")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, EXISTING_ACCOUNT)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let account = Decode!(
        &existing_account_res,
        ic_solana::rpc_client::RpcResult<Option<Account>>
    )
    .unwrap()
    .unwrap();

    assert!(account.is_some());

    let non_existing_account_res = agent
        .update(&canister_id, "sol_getAccountInfo")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, NON_EXISTING_ACCOUNT)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let account = Decode!(
        &non_existing_account_res,
        ic_solana::rpc_client::RpcResult<Option<Account>>
    )
    .unwrap();

    match account {
        Ok(_) => panic!("Expected an error"),
        Err(err) => match err {
            RpcError::Text(msg) => assert_eq!(
                msg,
                format!("AccountNotFound: pubkey={}", NON_EXISTING_ACCOUNT)
            ),
            _ => panic!("Expected a text error"),
        },
    }
}

#[tokio::test]
async fn test_get_transaction() {
    const EXISTING_SIGNATURE: &str =
        "3kxL8Qvp16kmVNiUkQSJ3zLvCJDK4qPZZ1ZL8W2VHeYoJUJnQ4VqMHFMNSmsGBq7rTfpe8cTzCopMSNRen6vGFt1";

    const NON_EXISTING_SIGNATURE: &str =
        "3kxL8Qvp16kmVNiUkQSJ3zLvCJDK4qPZZ1ZL8W2VHeYoJUJnQ4VqMHFMNSmsGBq7rTfpe8cTzCopMSNRen6vGFt2";

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

    let existing_account_res = agent
        .update(&canister_id, "sol_getTransaction")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, EXISTING_SIGNATURE)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let _ = Decode!(
        &existing_account_res,
        ic_solana::rpc_client::RpcResult<EncodedConfirmedTransactionWithStatusMeta>
    )
    .unwrap()
    .unwrap();

    let non_existing_account_res = agent
        .update(&canister_id, "sol_getTransaction")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, NON_EXISTING_SIGNATURE)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let tx = Decode!(
        &non_existing_account_res,
        ic_solana::rpc_client::RpcResult<EncodedConfirmedTransactionWithStatusMeta>
    )
    .unwrap();

    println!("TX: {:#?}", tx);
    todo!("Check the transaction after sol_getTransaction is fixed");
}

// milti_thread is needed for solana_client to work
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_send_raw_transaction() {
    // The amount to send from one account to the other, in lamports.
    const AMOUNT_TO_SEND: u64 = 1;

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

    let mut keypair1 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET1).unwrap();
    let mut keypair2 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET2).unwrap();
    let mut pubkey1 = keypair1.pubkey();
    let mut pubkey2 = keypair2.pubkey();

    // Requesting airdrop for keypair1 and keypair2,
    // so that they have some funds to send to each other.
    // If the airdrop fails, it means rate limit exceeded, but the test should still pass,
    // as the accounts should already have some funds.
    // NOTICE: If this test fails, it could potentially be due to lack of funds in the accounts.
    let _ = solana_client.request_airdrop(&pubkey1, sol_to_lamports(1.0));

    let _ = solana_client.request_airdrop(&pubkey2, sol_to_lamports(1.0));

    // Getting the pre balances of pubkey1 and pubkey2
    let call_result = agent
        .update(&canister_id, "sol_getBalance")
        .with_arg(encode_args((DEVNET_PROVIDER_ID, pubkey1.to_string())).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let mut pre_balance1 = Decode!(&call_result, ic_solana::rpc_client::RpcResult<u64>)
        .unwrap()
        .unwrap();

    let call_result = agent
        .update(&canister_id, "sol_getBalance")
        .with_arg(encode_args((DEVNET_PROVIDER_ID, pubkey2.to_string())).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let mut pre_balance2 = Decode!(&call_result, ic_solana::rpc_client::RpcResult<u64>)
        .unwrap()
        .unwrap();

    // Account with the most funds will send 1 lamport to the other account.
    if pre_balance1 < pre_balance2 {
        std::mem::swap(&mut keypair1, &mut keypair2);
        std::mem::swap(&mut pubkey1, &mut pubkey2);
        std::mem::swap(&mut pre_balance1, &mut pre_balance2);
    }

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
        .update(&canister_id, "sol_sendRawTransaction")
        .with_arg(encode_args((DEVNET_PROVIDER_ID, raw_tx)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let signature_str = Decode!(&signature, ic_solana::rpc_client::RpcResult<String>)
        .unwrap()
        .unwrap();

    let signature = Signature::from_str(&signature_str).unwrap();
    // Waiting for the transaction to be confirmed
    loop {
        if solana_client.confirm_transaction(&signature).unwrap() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    // Getting the tx details in order to confirm the transaction

    let call_result = agent
        .update(&canister_id, "sol_getTransaction")
        .with_arg(encode_args((DEVNET_PROVIDER_ID, signature_str)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let tx = Decode!(
        &call_result,
        ic_solana::rpc_client::RpcResult<EncodedConfirmedTransactionWithStatusMeta>
    )
    .unwrap();

    todo!("Check the transaction after sol_getTransaction is fixed");
}

// milti_thread is needed for solana_client to work
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "This test is not working, as the Schnorr signature is not currently supported in pocket-ic"]
async fn test_send_transaction() {
    // The amount to send from one account to the other, in lamports.
    const AMOUNT_TO_SEND: u64 = 1;

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

    let canister_id = init(&pic).await;

    let solana_client =
        solana_client::rpc_client::RpcClient::new(SOLANA_DEVNET_CLUSTER_URL.to_string());

    let call_result = agent
        .update(&canister_id, "sol_address")
        .with_arg(encode_one(DEVNET_PROVIDER_ID).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let canister_address = Decode!(&call_result, String).unwrap();

    let keypair1 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET1).unwrap();
    let pubkey1 = keypair1.pubkey();
    let canister_pubkey = Pubkey::from_str(&canister_address).unwrap();

    // Requesting airdrop for canister's address
    // so that it have some funds to send tx.
    // If the airdrop fails, it means rate limit exceeded, but the test should still pass,
    // as the account should already have some funds.
    // NOTICE: If this test fails, it could potentially be due to lack of funds in the accounts.
    let _ = solana_client.request_airdrop(&canister_pubkey, sol_to_lamports(1.0));

    let instruction = system_instruction::transfer(&canister_pubkey, &pubkey1, AMOUNT_TO_SEND);

    let instruction_str = bs58::encode(bincode::serialize(&instruction).unwrap()).into_string();

    let args = SendTransactionRequest {
        instructions: vec![instruction_str],
        recent_blockhash: None,
    };

    // Getting the pre balances of pubkey1 and pubkey2
    let call_result = agent
        .update(&canister_id, "sol_sendTransaction")
        .with_arg(encode_args((DEVNET_PROVIDER_ID, args)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let signature_str = Decode!(&call_result, ic_solana::rpc_client::RpcResult<String>)
        .unwrap()
        .unwrap();

    let signature = Signature::from_str(&signature_str).unwrap();

    // Waiting for the transaction to be confirmed
    loop {
        if solana_client.confirm_transaction(&signature).unwrap() {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    let call_result = agent
        .update(&canister_id, "sol_getTransaction")
        .with_arg(encode_args((DEVNET_PROVIDER_ID, signature_str)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let tx = Decode!(
        &call_result,
        ic_solana::rpc_client::RpcResult<EncodedConfirmedTransactionWithStatusMeta>
    )
    .unwrap();

    todo!("Check the transaction after sol_getTransaction is fixed");
}

#[tokio::test]
async fn test_request() {
    const METHOD: &str = "getBalance";
    const MAX_RESPONSE_BYTES: u64 = 200;
    const PARAMS: &str =
        r#"["AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY",{"minContextSlot":null}]"#;

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

    let canister_id = init(&pic).await;

    let call_result = agent
        .update(&canister_id, "request")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, METHOD, PARAMS, MAX_RESPONSE_BYTES)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let response = Decode!(&call_result, ic_solana::rpc_client::RpcResult<String>)
        .unwrap()
        .unwrap();

    todo!("Check the response after request is fixed");
}

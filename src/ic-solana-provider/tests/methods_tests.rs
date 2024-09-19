mod common;

use {
    crate::common::init,
    candid::{encode_args, encode_one, Decode},
    common::{
        BASIC_IDENTITY, DEVNET_PROVIDER_ID, MAINNET_PROVIDER_ID, PUBKEY1, SECRET1, SECRET2,
        SOLANA_DEVNET_CLUSTER_URL,
    },
    core::panic,
    ic_agent::Agent,
    ic_solana::{
        rpc_client::RpcError,
        types::{
            Account, CandidValue, CommitmentConfig,
            TaggedEncodedConfirmedTransactionWithStatusMeta, TaggedEncodedTransaction,
            TaggedUiMessage, TransactionStatus, UiTokenAmount,
        },
    },
    ic_solana_provider::types::SendTransactionRequest,
    pocket_ic::PocketIcBuilder,
    solana_sdk::{
        bs58, native_token::sol_to_lamports, signature::Signature, signer::Signer,
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

    const TRANSACTION_RESPONSE_SIZE_ESTIMATE: u64 = 10500;

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
        .with_arg(
            encode_args((
                MAINNET_PROVIDER_ID,
                EXISTING_SIGNATURE,
                TRANSACTION_RESPONSE_SIZE_ESTIMATE,
            ))
            .unwrap(),
        )
        .call_and_wait()
        .await
        .unwrap();

    let _ = Decode!(
        &existing_account_res,
        ic_solana::rpc_client::RpcResult<TaggedEncodedConfirmedTransactionWithStatusMeta>
    )
    .unwrap()
    .unwrap();
}

// `multi_thread` is required for a solana client to work
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_send_raw_transaction() {
    // The amount to send from one account to the other, in lamports.
    const AMOUNT_TO_SEND: u64 = 1;
    const FEE: u64 = 5000;

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
    let _ = agent
        .update(&canister_id, "sol_requestAirdrop")
        .with_arg(
            encode_args((
                DEVNET_PROVIDER_ID,
                pubkey1.to_string(),
                sol_to_lamports(1.0),
            ))
            .unwrap(),
        )
        .call_and_wait()
        .await
        .unwrap();

    let _ = agent
        .update(&canister_id, "sol_requestAirdrop")
        .with_arg(
            encode_args((
                DEVNET_PROVIDER_ID,
                pubkey2.to_string(),
                sol_to_lamports(1.0),
            ))
            .unwrap(),
        )
        .call_and_wait()
        .await
        .unwrap();

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

    // Getting the latest blockhash
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
        ic_solana::rpc_client::RpcResult<TaggedEncodedConfirmedTransactionWithStatusMeta>
    )
    .unwrap()
    .unwrap();

    let pre_balances = tx
        .transaction
        .meta
        .as_ref()
        .unwrap()
        .pre_balances
        .as_slice();

    let post_balances = tx
        .transaction
        .meta
        .as_ref()
        .unwrap()
        .post_balances
        .as_slice();

    let account_keys = if let TaggedEncodedTransaction::Json(ref json) = tx.transaction.transaction
    {
        if let TaggedUiMessage::Raw(ref message_raw) = json.message {
            &message_raw.account_keys
        } else {
            panic!("Expected a raw message");
        }
    } else {
        panic!("Expected a json transaction");
    };

    assert_eq!(
        account_keys,
        &vec![
            pubkey1.to_string(),
            pubkey2.to_string(),
            "11111111111111111111111111111111".to_string()
        ]
    );

    assert_eq!(post_balances[0], pre_balances[0] - AMOUNT_TO_SEND - FEE);
    assert_eq!(post_balances[1], pre_balances[1] + AMOUNT_TO_SEND);
    assert_eq!(post_balances[2], pre_balances[2]);
}

#[tokio::test]
async fn test_send_transaction() {
    // The amount to send from one account to the other, in lamports.
    const AMOUNT_TO_SEND: u64 = 1;
    const FEE: u64 = 5000;

    let mut pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_ii_subnet()
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
        .update(&canister_id, "sol_address")
        .with_arg(encode_one(DEVNET_PROVIDER_ID).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let canister_address = Decode!(&call_result, String).unwrap();

    let keypair1 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET1).unwrap();
    let pubkey1 = keypair1.pubkey();
    let canister_pubkey = solana_sdk::pubkey::Pubkey::from_str(&canister_address).unwrap();

    // Requesting airdrop for canister's address
    // so that it have some funds to send tx.
    // If the airdrop fails, it means rate limit exceeded, but the test should still pass,
    // as the account should already have some funds.
    // NOTICE: If this test fails, it could potentially be due to lack of funds in the accounts.
    // Requesting airdrop for keypair1 and keypair2,
    // so that they have some funds to send to each other.
    // If the airdrop fails, it means rate limit exceeded, but the test should still pass,
    // as the accounts should already have some funds.
    // NOTICE: If this test fails, it could potentially be due to lack of funds in the accounts.
    let _ = agent
        .update(&canister_id, "sol_requestAirdrop")
        .with_arg(
            encode_args((
                DEVNET_PROVIDER_ID,
                canister_address.to_string(),
                sol_to_lamports(1.0),
            ))
            .unwrap(),
        )
        .call_and_wait()
        .await
        .unwrap();

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

    // let solana_client =
    //     solana_client::rpc_client::RpcClient::new(SOLANA_DEVNET_CLUSTER_URL.to_string());
    // let signature = Signature::from_str(&signature_str).unwrap();

    // Waiting for the transaction to be confirmed
    loop {
        let call_result = agent
            .update(&canister_id, "sol_getSignatureStatuses")
            .with_arg(encode_args((DEVNET_PROVIDER_ID, vec![&signature_str])).unwrap())
            .call_and_wait()
            .await
            .unwrap();

        let res = Decode!(
            &call_result,
            ic_solana::rpc_client::RpcResult<Vec<Option<TransactionStatus>>>
        )
        .unwrap()
        .unwrap();

        let tx_status = res.first().unwrap();

        if let Some(tx_status) = tx_status {
            if tx_status.status.is_ok()
                && tx_status.satisfies_commitment(CommitmentConfig::finalized())
            {
                break;
            }
        }

        // if solana_client.confirm_transaction(&signature).unwrap() {
        //     break;
        // }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    let call_result = agent
        .update(&canister_id, "sol_getTransaction")
        .with_arg(encode_args((DEVNET_PROVIDER_ID, &signature_str)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let tx = Decode!(
        &call_result,
        ic_solana::rpc_client::RpcResult<TaggedEncodedConfirmedTransactionWithStatusMeta>
    )
    .unwrap()
    .unwrap();

    let pre_balances = tx
        .transaction
        .meta
        .as_ref()
        .unwrap()
        .pre_balances
        .as_slice();

    let post_balances = tx
        .transaction
        .meta
        .as_ref()
        .unwrap()
        .post_balances
        .as_slice();

    let account_keys = if let TaggedEncodedTransaction::Json(ref json) = tx.transaction.transaction
    {
        if let TaggedUiMessage::Raw(ref message_raw) = json.message {
            &message_raw.account_keys
        } else {
            panic!("Expected a raw message");
        }
    } else {
        panic!("Expected a json transaction");
    };

    assert_eq!(
        account_keys,
        &vec![
            canister_address,
            pubkey1.to_string(),
            "11111111111111111111111111111111".to_string()
        ]
    );

    assert_eq!(post_balances[0], pre_balances[0] - AMOUNT_TO_SEND - FEE);
    assert_eq!(post_balances[1], pre_balances[1] + AMOUNT_TO_SEND);
    assert_eq!(post_balances[2], pre_balances[2]);
}

#[tokio::test]
async fn test_request() {
    const METHOD: &str = "getBalance";
    const MAX_RESPONSE_BYTES: u64 = 200;

    let params = serde_json::json!(
        ["AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY",{"minContextSlot":null}]
    );
    let params = CandidValue(params);

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
        .with_arg(encode_args((MAINNET_PROVIDER_ID, METHOD, params, MAX_RESPONSE_BYTES)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let response = Decode!(&call_result, ic_solana::rpc_client::RpcResult<String>)
        .unwrap()
        .unwrap();

    println!("{}", response);
}

#[tokio::test]
async fn test_request_airdrop() {
    const AIRDROP_AMOUNT_LAMPORTS: u64 = 100000000;
    let pubkey = solana_sdk::pubkey::Pubkey::new_unique().to_string();

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
        .update(&canister_id, "sol_getBalance")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, &pubkey)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let balance_before = Decode!(&call_result, ic_solana::rpc_client::RpcResult<u64>)
        .unwrap()
        .unwrap();

    let call_result = agent
        .update(&canister_id, "sol_requestAirdrop")
        .with_arg(
            encode_args((
                DEVNET_PROVIDER_ID,
                PUBKEY1.to_string(),
                AIRDROP_AMOUNT_LAMPORTS,
            ))
            .unwrap(),
        )
        .call_and_wait()
        .await
        .unwrap();

    let response = Decode!(&call_result, ic_solana::rpc_client::RpcResult<String>).unwrap();

    if let Err(err) = response {
        if err.to_string().contains("airdrop limit") {
            // If the airdrop limit is reached, the test should still pass
            return;
        }
        panic!("Airdrop error: {}", err);
    }

    // If the air drop was successful, the balance should be increased by AIRDROP_AMOUNT_LAMPROTS
    let call_result = agent
        .update(&canister_id, "sol_getBalance")
        .with_arg(encode_args((MAINNET_PROVIDER_ID, pubkey)).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    let balance_after = Decode!(&call_result, ic_solana::rpc_client::RpcResult<u64>)
        .unwrap()
        .unwrap();

    assert_eq!(balance_after - balance_before, AIRDROP_AMOUNT_LAMPORTS);
}

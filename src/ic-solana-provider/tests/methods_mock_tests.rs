mod common;

use {
    crate::common::init,
    candid::{encode_args, encode_one},
    common::{decode_raw_wasm_result, fast_forward, MAINNET_PROVIDER_ID, SECRET1, USER_PRINCIPAL},
    ic_solana::types::{
        Account, EncodedConfirmedTransactionWithStatusMeta,
        TaggedEncodedConfirmedTransactionWithStatusMeta, UiTokenAmount,
    },
    ic_solana_provider::types::SendTransactionRequest,
    pocket_ic::{
        common::rest::{CanisterHttpReply, CanisterHttpResponse, MockCanisterHttpResponse},
        PocketIcBuilder,
    },
    serde_json::Value,
    solana_sdk::{bs58, pubkey::Pubkey, signer::Signer, system_instruction},
    std::str::FromStr,
};

#[tokio::test]
async fn test_get_balance_mock() {
    const ACCOUNT: &str = "AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY";
    const RESPONSE: &[u8] = b"{\"jsonrpc\":\"2.0\",\"result\":{\"context\":{\"apiVersion\":\"2.0.8\",\"slot\":324152736},\"value\":228},\"id\":0}";
    const EXPECTED: u64 = 228;

    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nns_subnet()
        .build_async()
        .await;

    let canister_id = init(&pic).await;

    let call = pic
        .submit_call(
            canister_id,
            *USER_PRINCIPAL,
            "sol_getBalance",
            encode_args((MAINNET_PROVIDER_ID, ACCOUNT)).unwrap(),
        )
        .await
        .unwrap();

    fast_forward(&pic, 5).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.first().unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: RESPONSE.to_vec(),
        }),
        additional_responses: None,
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<u64>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    assert_eq!(result.unwrap(), EXPECTED);
}

#[tokio::test]
async fn test_get_token_balance_mock() {
    const ACCOUNT: &str = "7fUAJdStEuGbc3sM84cKRL6yYaaSstyLSU4ve5oovLS7";
    const RESPONSE:&[u8]=b"{\"jsonrpc\":\"2.0\",\"result\":{\"context\":{\"slot\":1114},\"value\":{\"amount\":\"9864\",\"decimals\":2,\"uiAmount\":98.64,\"uiAmountString\":\"98.64\"}},\"id\":1}";
    let expected: UiTokenAmount = UiTokenAmount {
        amount: "9864".to_string(),
        decimals: 2,
        ui_amount: Some(98.64),
        ui_amount_string: "98.64".to_string(),
    };

    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nns_subnet()
        .build_async()
        .await;

    let canister_id = init(&pic).await;

    let call = pic
        .submit_call(
            canister_id,
            *USER_PRINCIPAL,
            "sol_getTokenBalance",
            encode_args((MAINNET_PROVIDER_ID, ACCOUNT)).unwrap(),
        )
        .await
        .unwrap();

    fast_forward(&pic, 4).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.first().unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: RESPONSE.to_vec(),
        }),
        additional_responses: None,
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<UiTokenAmount>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let token_amount = result.expect("RPC error occurred");

    assert_eq!(token_amount, expected);
}

#[tokio::test]
async fn test_get_latest_blockhash_mock() {
    const RESPONSE: &[u8] = b"{\"jsonrpc\":\"2.0\",\"result\":{\"context\":{\"slot\":2792},\"value\":{\"blockhash\":\"EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N\",\"lastValidBlockHeight\":3090}},\"id\":1}";

    const EXPECTED: &str = "EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N";

    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nns_subnet()
        .build_async()
        .await;

    let canister_id = init(&pic).await;

    let call = pic
        .submit_call(
            canister_id,
            *USER_PRINCIPAL,
            "sol_getLatestBlockhash",
            encode_one(MAINNET_PROVIDER_ID).unwrap(),
        )
        .await
        .unwrap();

    fast_forward(&pic, 4).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.first().unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: RESPONSE.to_vec(),
        }),
        additional_responses: None,
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<String>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let blockhash = result.expect("RPC error occurred");

    assert_eq!(blockhash, EXPECTED);
}

#[tokio::test]
async fn test_get_account_info_mock() {
    const RESPONSE: &[u8] = b"{\"jsonrpc\":\"2.0\",\"result\":{\"context\":{\"apiVersion\":\"1.18.23\",\"slot\":288920130},\"value\":{\"data\":[\"DK9MyTErE4QfyzYKUKJ3hm913rZNR7cJkvmNXmAxRQrAuhyoATY3vEBy8iD48RmTuQEHSNabA98xwY9sbUUTn7Trxf27sYrrZDqyKN6ra1Di6ni\",\"base58\"],\"executable\":false,\"lamports\":87918840915,\"owner\":\"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\",\"rentEpoch\":18446744073709551615,\"space\":82}},\"id\":1}";
    const ACCOUNT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

    let expected: Option<Account> = Some(Account {
        lamports: 87918840915,
        data: vec![
            0x01, 0x00, 0x00, 0x00, 0x05, 0xea, 0x9c, 0xf1, 0x6c, 0xe4, 0x11, 0x98, 0xf1, 0xa4,
            0x99, 0x37, 0xc8, 0x8c, 0x37, 0x0a, 0x94, 0xd4, 0xaf, 0xff, 0x89, 0xb5, 0xba, 0xcb,
            0x8e, 0xf4, 0x5e, 0x63, 0x24, 0xbb, 0x78, 0xf7, 0x7a, 0x9f, 0x13, 0x85, 0xe3, 0xb6,
            0x06, 0x00, 0x06, 0x01, 0x01, 0x00, 0x00, 0x00, 0x05, 0xea, 0x9c, 0xf1, 0x6c, 0xe4,
            0x11, 0x98, 0xf1, 0xa4, 0x99, 0x37, 0xc8, 0x8c, 0x37, 0x0a, 0x94, 0xd4, 0xaf, 0xff,
            0x89, 0xb5, 0xba, 0xcb, 0x8e, 0xf4, 0x5e, 0x63, 0x24, 0xbb, 0x78, 0xf7,
        ],
        owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            .parse()
            .unwrap(),
        executable: false,
        rent_epoch: 18446744073709551615,
    });

    let pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .build_async()
        .await;

    let canister_id = init(&pic).await;

    let call = pic
        .submit_call(
            canister_id,
            *USER_PRINCIPAL,
            "sol_getAccountInfo",
            encode_args((MAINNET_PROVIDER_ID, ACCOUNT)).unwrap(),
        )
        .await
        .unwrap();

    fast_forward(&pic, 4).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.first().unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: RESPONSE.to_vec(),
        }),
        additional_responses: None,
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<Option<Account>>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let account = result.expect("RPC error occurred");

    assert_eq!(account, expected);
}

#[tokio::test]
async fn test_get_transaction_mock() {
    const SIGNATURE: &str =
        "3kxL8Qvp16kmVNiUkQSJ3zLvCJDK4qPZZ1ZL8W2VHeYoJUJnQ4VqMHFMNSmsGBq7rTfpe8cTzCopMSNRen6vGFt1";

    const RESPONSE_STR: &str = r###"
{
  "jsonrpc": "2.0",
  "result": {
    "blockTime": 1725954458,
    "meta": {
      "computeUnitsConsumed": 142936,
      "err": null,
      "fee": 7400,
      "innerInstructions": [
        {
          "index": 1,
          "instructions": [
            {
              "accounts": [
                3,
                5,
                19
              ],
              "data": "3ay4cmSa9v5u",
              "programIdIndex": 20,
              "stackHeight": 2
            }
          ]
        },
        {
          "index": 2,
          "instructions": [
            {
              "accounts": [
                9,
                6,
                22
              ],
              "data": "3Dc4pim41vmd",
              "programIdIndex": 20,
              "stackHeight": 2
            }
          ]
        },
        {
          "index": 3,
          "instructions": [
            {
              "accounts": [
                5,
                10,
                0
              ],
              "data": "3KJnQat1Fiib",
              "programIdIndex": 20,
              "stackHeight": 2
            }
          ]
        },
        {
          "index": 4,
          "instructions": [
            {
              "accounts": [
                6,
                4,
                0
              ],
              "data": "3qottyySieFh",
              "programIdIndex": 20,
              "stackHeight": 2
            }
          ]
        }
      ],
      "loadedAddresses": {
        "readonly": [],
        "writable": []
      },
      "logMessages": [
        "Program ComputeBudget111111111111111111111111111111 invoke [1]",
        "Program ComputeBudget111111111111111111111111111111 success",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb invoke [1]",
        "Program log: Instruction: SettleFunds",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
        "Program log: Instruction: Transfer",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 782609 compute units",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
        "Program data: CjLwde1D5umUJll68E8UShvGApTsIHaTw8TYxQEDMHFw9zeM8g2hCYCXd/YBAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb consumed 23898 of 799850 compute units",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb success",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb invoke [1]",
        "Program log: Instruction: SettleFunds",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
        "Program log: Instruction: Transfer",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 758711 compute units",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
        "Program data: CjLwde1D5unNP4yqTqwfPi7MahPay9UVaVOsvuDYA+uA4QhOto5JcADgZzUAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb consumed 23898 of 775952 compute units",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb success",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb invoke [1]",
        "Program log: Instruction: CancelAllAndPlaceOrders",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
        "Program log: Instruction: Transfer",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 712315 compute units",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb consumed 46738 of 752054 compute units",
        "Program return: opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb AgAAAAEVi/z//////1rMDgAAAAAAARSL/P//////hUEPAAAAAAA=",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb success",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb invoke [1]",
        "Program log: Instruction: CancelAllAndPlaceOrders",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
        "Program log: Instruction: Transfer",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 664063 compute units",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb consumed 48252 of 705316 compute units",
        "Program return: opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb AgAAAAFAavj//////1bNDgAAAAAAAT9q+P//////iEIPAAAAAAA=",
        "Program opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb success"
      ],
      "postBalances": [
        588111184,
        9688420,
        6793060,
        2039380,
        2039380,
        2039280,
        2039280,
        9688320,
        6792960,
        2039280,
        2039280,
        633916800,
        633916800,
        636255360,
        633916900,
        633916900,
        636255460,
        1,
        1141440,
        0,
        934087680,
        1,
        0
      ],
      "postTokenBalances": [
        {
          "accountIndex": 3,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "3718000000",
            "decimals": 6,
            "uiAmount": 3718.0,
            "uiAmountString": "3718"
          }
        },
        {
          "accountIndex": 4,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "19913589900",
            "decimals": 6,
            "uiAmount": 19913.5899,
            "uiAmountString": "19913.5899"
          }
        },
        {
          "accountIndex": 5,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "158303",
            "decimals": 6,
            "uiAmount": 0.158303,
            "uiAmountString": "0.158303"
          }
        },
        {
          "accountIndex": 6,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "235942",
            "decimals": 6,
            "uiAmount": 0.235942,
            "uiAmountString": "0.235942"
          }
        },
        {
          "accountIndex": 9,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "23000000",
            "decimals": 6,
            "uiAmount": 23.0,
            "uiAmountString": "23"
          }
        },
        {
          "accountIndex": 10,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "262364494446",
            "decimals": 6,
            "uiAmount": 262364.494446,
            "uiAmountString": "262364.494446"
          }
        }
      ],
      "preBalances": [
        588118584,
        9688420,
        6793060,
        2039380,
        2039380,
        2039280,
        2039280,
        9688320,
        6792960,
        2039280,
        2039280,
        633916800,
        633916800,
        636255360,
        633916900,
        633916900,
        636255460,
        1,
        1141440,
        0,
        934087680,
        1,
        0
      ],
      "preTokenBalances": [
        {
          "accountIndex": 3,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "12148000000",
            "decimals": 6,
            "uiAmount": 12148.0,
            "uiAmountString": "12148"
          }
        },
        {
          "accountIndex": 4,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "19017494707",
            "decimals": 6,
            "uiAmount": 19017.494707,
            "uiAmountString": "19017.494707"
          }
        },
        {
          "accountIndex": 5,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "313857",
            "decimals": 6,
            "uiAmount": 0.313857,
            "uiAmountString": "0.313857"
          }
        },
        {
          "accountIndex": 6,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "331135",
            "decimals": 6,
            "uiAmount": 0.331135,
            "uiAmountString": "0.331135"
          }
        },
        {
          "accountIndex": 9,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "919000000",
            "decimals": 6,
            "uiAmount": 919.0,
            "uiAmountString": "919"
          }
        },
        {
          "accountIndex": 10,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": {
            "amount": "253934338892",
            "decimals": 6,
            "uiAmount": 253934.338892,
            "uiAmountString": "253934.338892"
          }
        }
      ],
      "returnData": {
        "data": [
          "AgAAAAFAavj//////1bNDgAAAAAAAT9q+P//////iEIPAAAAAAA=",
          "base64"
        ],
        "programId": "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb"
      },
      "rewards": [],
      "status": {
        "Ok": null
      }
    },
    "slot": 288918198,
    "transaction": {
      "message": {
        "accountKeys": [
          "Cs1CmcYgRorcAwhAeWp5BzqMbK92ea7bSTE8sZWtAXfx",
          "AyKFqUZd6FNt2jMeg7Bhbu4cS8RKMQxp2owBtjqVG1gU",
          "ECWHdhxPX2Vx299xvB1Dd5aTegGJmFcXWhrESftsKkRc",
          "8sxE2FYK3dzkXQ4oy8phVoJBZU9jgvpUuJWAhwDLtmfP",
          "Cb6wY5fpawswXhSS5hqCE3xW3Qw2GjA7soxjLNnarvCd",
          "458uFwc7urgyWwed7wRVLvY9h2eTxAKSeSA7M2f5hj5e",
          "eWmZj4TDx4tzgn49texEgDzABTzkrrQzgSiDpv3DSL5",
          "EpCnL2VMaSTgr7VPVaA2f69Y4iB5xAPivNAPgg7uHWab",
          "ArAsfvhAKb9F7H8SWhsZzf3ncRdzs7kMSaw1fxiyg9P2",
          "7sViAX3S7rzd3fbN56AsDEzehbgmi8We3NHBwbb7UHfU",
          "8uT1JEq9bxcXPNtgBJeXzQgDwojxkjxy1PC8uSEQuSBG",
          "BbZkRsLdtqJ8wsNkcGCkbJ3XN4ndR4PEFKGjyMWs4E9T",
          "5mgHN3wagFGSnPHN9VaDJp2BdbQonQ1bJSPow1Dcav3u",
          "5mGwZCeVcTjoWfs2uDgLEaP8H9F1GZwhPiKXucPyS5JY",
          "HGaofFyMA9xeePf7if3LtnhbciPuYcCKbqadNKka7MSq",
          "H1f8y3ujiHA6Uk9eBjhgmdCEq38rvqbWj2vjegLv5ryk",
          "9YacbvkSFuQ1nNtzshrARnM7HdPwidpc4MWySU9DMEKt",
          "ComputeBudget111111111111111111111111111111",
          "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb",
          "Hwr7Av8kg5qJtMkAyiEmMkAiTWr6ZULmyE32hqmSSCSU",
          "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "11111111111111111111111111111111",
          "6NUn63sRLS2oEHtAeaAMMMFTdmfPUX78McT47Hmt2t9i"
        ],
        "addressTableLookups": [],
        "header": {
          "numReadonlySignedAccounts": 0,
          "numReadonlyUnsignedAccounts": 6,
          "numRequiredSignatures": 1
        },
        "instructions": [
          {
            "accounts": [],
            "data": "3kF1YY3pexPH",
            "programIdIndex": 17,
            "stackHeight": null
          },
          {
            "accounts": [
              0,
              0,
              1,
              2,
              19,
              3,
              4,
              5,
              6,
              18,
              20,
              21
            ],
            "data": "grMARW36kxQ",
            "programIdIndex": 18,
            "stackHeight": null
          },
          {
            "accounts": [
              0,
              0,
              7,
              8,
              22,
              9,
              10,
              6,
              5,
              18,
              20,
              21
            ],
            "data": "grMARW36kxQ",
            "programIdIndex": 18,
            "stackHeight": null
          },
          {
            "accounts": [
              0,
              7,
              18,
              5,
              6,
              8,
              11,
              12,
              13,
              10,
              9,
              21,
              21,
              20
            ],
            "data": "s6Pv7kdeSzJWnWia6ULkQwqtfTkJQb2daW2MXMaXTXMxfzqmYwZcxYTHhRQHMkLj3mDnVy3uzZt8VY7DVMfw5P9Hmu",
            "programIdIndex": 18,
            "stackHeight": null
          },
          {
            "accounts": [
              0,
              1,
              18,
              6,
              5,
              2,
              14,
              15,
              16,
              4,
              3,
              21,
              21,
              20
            ],
            "data": "s6Pv7kdeSzJWnWia6UGWraJGzKaE5xs9pKLo8fMPtZn4uXTtWwJLmiTNPHVEiXNrWZjrCH5g6gxfZxwb7UBr2tUVHu",
            "programIdIndex": 18,
            "stackHeight": null
          }
        ],
        "recentBlockhash": "4Piqad6azGRvWcKS2xqv9ThjqGiSLrAycye5PztpMDYP"
      },
      "signatures": [
        "3kxL8Qvp16kmVNiUkQSJ3zLvCJDK4qPZZ1ZL8W2VHeYoJUJnQ4VqMHFMNSmsGBq7rTfpe8cTzCopMSNRen6vGFt1"
      ]
    },
    "version": 0
  },
  "id": 1
}
    "###;

    let response: &[u8] = RESPONSE_STR.as_bytes();

    let value: Value = serde_json::from_str::<Value>(RESPONSE_STR)
        .unwrap()
        .get("result")
        .unwrap()
        .clone();

    let expected: EncodedConfirmedTransactionWithStatusMeta =
        serde_json::from_value(value).expect("Failed to parse response");
    let expected = expected.into();

    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nns_subnet()
        .build_async()
        .await;

    let canister_id = init(&pic).await;

    let call = pic
        .submit_call(
            canister_id,
            *USER_PRINCIPAL,
            "sol_getTransaction",
            encode_args((MAINNET_PROVIDER_ID, SIGNATURE)).unwrap(),
        )
        .await
        .unwrap();

    fast_forward(&pic, 4).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.first().unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: response.to_vec(),
        }),
        additional_responses: None,
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (
        ic_solana::rpc_client::RpcResult<TaggedEncodedConfirmedTransactionWithStatusMeta>,
    ) = decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let tx = result.expect("RPC error occurred");

    assert_eq!(tx, expected);
}

#[tokio::test]
async fn test_send_raw_transaction_mock() {
    const SEND_RAW_TRANSACTION_REQUEST: &str = r###"5Pqoict6REKLCaPAWadjLm2Vj5JKwvEVwJFfxYP8JiuC8FDUdxVhWUebF4gEcCR5mao8WizwGEKYRTC5X47TEN4HqUqJjcqSx1FMtyjvF37rXGqqUWm69GwxCTuzuEEFvZcguoBLemroUSmZimkPiE33J7i6RpRmZzLNJ2owzsSYHKZr5NHXaBN5hHiyzuDAakVqK1VRdyTKsVrA53xuLom7uHb7oMvYKPPWpuEaMxigWHcjfq3782no5gK1csSMEdaQavh4vU6CCr4kroM4X5KHd5qmz3TXGEEum"###;
    const SEND_RAW_TRANSACTION_RESPONSE: &[u8] = br###"{"jsonrpc":"2.0","result":"418jqKR8c5pFDNo7h9vdyFaCdhQKr9z3rmEvkaz8Fcd34eSsAhsvsHkpffYmu5YfsvHWu1uFGo48Dbm3no8cFobM","id":0}"###;
    const EXPECTED: &str =
        "418jqKR8c5pFDNo7h9vdyFaCdhQKr9z3rmEvkaz8Fcd34eSsAhsvsHkpffYmu5YfsvHWu1uFGo48Dbm3no8cFobM";

    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nns_subnet()
        .build_async()
        .await;

    let canister_id = init(&pic).await;

    let call = pic
        .submit_call(
            canister_id,
            *USER_PRINCIPAL,
            "sol_sendRawTransaction",
            encode_args((MAINNET_PROVIDER_ID, SEND_RAW_TRANSACTION_REQUEST)).unwrap(),
        )
        .await
        .unwrap();

    fast_forward(&pic, 4).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.first().unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: SEND_RAW_TRANSACTION_RESPONSE.to_vec(),
        }),
        additional_responses: None,
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<String>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let signature = result.expect("RPC error occurred");

    assert_eq!(signature, EXPECTED.to_string());
}

#[tokio::test]
async fn test_send_transaction_mock() {
    const AMOUNT_TO_SEND: u64 = 1;
    const SEND_TRANSACTION_RESPONSE: &[u8] = br###"{"jsonrpc":"2.0","result":"2EanSSkn5cjv9DVKik5gtBkN1wwbV1TAXQQ5yu2RTPGwgrhEywVAQR2veu895uCDzvYwWZe6vD1Bcn8s7r22W17w","id":0}"###;
    const LATEST_BLOCKHASH_RESPONSE: &[u8] = br###"{ "jsonrpc": "2.0", "result": { "context": { "slot": 2792 }, "value": { "blockhash": "EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N", "lastValidBlockHeight": 3090 } }, "id": 1 }"###;
    const EXPECTED: &str =
        "2EanSSkn5cjv9DVKik5gtBkN1wwbV1TAXQQ5yu2RTPGwgrhEywVAQR2veu895uCDzvYwWZe6vD1Bcn8s7r22W17w";

    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_ii_subnet()
        .with_nns_subnet()
        .build_async()
        .await;

    let canister_id = init(&pic).await;

    let call_result = pic
        .update_call(
            canister_id,
            *USER_PRINCIPAL,
            "sol_address",
            encode_one(()).unwrap(),
        )
        .await
        .unwrap();

    let (canister_address,): (String,) = decode_raw_wasm_result(&call_result).unwrap();

    let keypair1 = solana_sdk::signer::keypair::Keypair::from_bytes(&SECRET1).unwrap();
    let pubkey1 = keypair1.pubkey();
    let canister_pubkey = Pubkey::from_str(&canister_address).unwrap();

    let instruction = system_instruction::transfer(&canister_pubkey, &pubkey1, AMOUNT_TO_SEND);

    let instruction_str = bs58::encode(bincode::serialize(&instruction).unwrap()).into_string();

    let args = SendTransactionRequest {
        instructions: vec![instruction_str],
        recent_blockhash: None,
    };

    let call = pic
        .submit_call(
            canister_id,
            *USER_PRINCIPAL,
            "sol_sendTransaction",
            encode_args((MAINNET_PROVIDER_ID, args)).unwrap(),
        )
        .await
        .unwrap();

    fast_forward(&pic, 4).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.first().unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: LATEST_BLOCKHASH_RESPONSE.to_vec(),
        }),
        additional_responses: None,
    };

    pic.mock_canister_http_response(mock).await;

    fast_forward(&pic, 10).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.get(0).unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: SEND_TRANSACTION_RESPONSE.to_vec(),
        }),
        additional_responses: None,
    };

    pic.mock_canister_http_response(mock).await;

    fast_forward(&pic, 4).await;

    let (result,): (ic_solana::rpc_client::RpcResult<String>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let signature = result.expect("RPC error occurred");

    assert_eq!(signature, EXPECTED.to_string());
}

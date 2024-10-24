mod common;

use {
    crate::common::init,
    candid::{encode_args, encode_one},
    common::{decode_raw_wasm_result, fast_forward, MAINNET_PROVIDER_ID, SECRET1, USER_PRINCIPAL},
    ic_solana::types::{
        Account, EncodedConfirmedTransactionWithStatusMeta, TaggedEncodedConfirmedTransactionWithStatusMeta,
        TaggedRpcKeyedAccount, TaggedRpcTokenAccountBalance, TaggedUiConfirmedBlock, TokenAccountsFilter,
        TransactionDetails, UiTokenAmount,
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<u64>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    assert_eq!(result.unwrap(), EXPECTED);

    pic.drop().await;
}

#[tokio::test]
async fn test_get_block_mock() {
    const SLOT: u64 = 10_000;
    const MAX_RESPONSE_BYTES: u64 = 1_500_000;

    const RESPONSE: &[u8] = br#"{"jsonrpc":"2.0","result":{"blockHeight":null,"blockTime":null,"blockhash":"FNy3uy9b9EMupvMpzG6Waqtbpt5Hto3naW2NnDwL1eYq","parentSlot":9999,"previousBlockhash":"8sxZZjKCz85m5ZsgPzhE1z5tSqzjcsXM45M78sPb55ET","rewards":[],"transactions":[{"meta":null,"transaction":{"message":{"accountKeys":["7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2","4785anyR2rYSas6cQGHtykgzwYEtChvFYhcEgdDw3gGL","SysvarS1otHashes111111111111111111111111111","SysvarC1ock11111111111111111111111111111111","Vote111111111111111111111111111111111111111"],"header":{"numReadonlySignedAccounts":0,"numReadonlyUnsignedAccounts":3,"numRequiredSignatures":1},"instructions":[{"accounts":[1,2,3,0],"data":"37u9WtQpcm6ULa3VpnRF6CqaEftTmTtPNaoq5eaAxgTubYiNR7Jf3wpSvqhp2P7c7Dx68kAb","programIdIndex":4,"stackHeight":null}],"recentBlockhash":"8sxZZjKCz85m5ZsgPzhE1z5tSqzjcsXM45M78sPb55ET"},"signatures":["4Kd11S9XoL2g9tkh45hW6JgnPwVhSCmFDxSJnPeXKQHYoLoTYf5aL3JU7r8DjiJ4EBvAPTwpCJdVaArSzQMuEWqA"]}},{"meta":null,"transaction":{"message":{"accountKeys":["DE1bawNcRJB9rVm3buyMVfr8mBEoyyu73NBovf2oXJsJ","8XgHUtBRY6qePVYERxosyX3MUq8NQkjtmFDSzQ2WpHTJ","SysvarS1otHashes111111111111111111111111111","SysvarC1ock11111111111111111111111111111111","Vote111111111111111111111111111111111111111"],"header":{"numReadonlySignedAccounts":0,"numReadonlyUnsignedAccounts":3,"numRequiredSignatures":1},"instructions":[{"accounts":[1,2,3,0],"data":"37u9WtQpcm6ULa3VpnRF6CqaEftTmTtPNaoq5eaAxgTubYiNR7Jf3wpSvqhp2P7c7Dx68kAb","programIdIndex":4,"stackHeight":null}],"recentBlockhash":"8sxZZjKCz85m5ZsgPzhE1z5tSqzjcsXM45M78sPb55ET"},"signatures":["2jY6X9aLUj45xrnFvjULjhMcWoMJwqECKsCLE9Nnhch4DDZajZLVB7pGzXXGpqDRcbpNDM1eG5k8VxQxGxPKnD6H"]}},{"meta":null,"transaction":{"message":{"accountKeys":["CakcnaRDHka2gXyfbEd2d3xsvkJkqsLw2akB3zsN1D2S","9bRDrYShoQ77MZKYTMoAsoCkU7dAR24mxYCBjXLpfEJx","SysvarS1otHashes111111111111111111111111111","SysvarC1ock11111111111111111111111111111111","Vote111111111111111111111111111111111111111"],"header":{"numReadonlySignedAccounts":0,"numReadonlyUnsignedAccounts":3,"numRequiredSignatures":1},"instructions":[{"accounts":[1,2,3,0],"data":"37u9WtQpcm6ULa3VpnRF6CqaEftTmTtPNaoq5eaAxgTubYiNR7Jf3wpSvqhp2P7c7Dx68kAb","programIdIndex":4,"stackHeight":null}],"recentBlockhash":"8sxZZjKCz85m5ZsgPzhE1z5tSqzjcsXM45M78sPb55ET"},"signatures":["9b4aincvJ7YgC9MFUZQP8TpT3hZ6cqvAfSVDBaw4TZ44ZpkZHR29aqcQ36yRy5gYvTS372o21C6PKUZNoRFAqEE"]}},{"meta":null,"transaction":{"message":{"accountKeys":["GdnSyH3YtwcxFvQrVVJMm1JhTS4QVX7MFsX56uJLUfiZ","sCtiJieP8B3SwYnXemiLpRFRR8KJLMtsMVN25fAFWjW","SysvarS1otHashes111111111111111111111111111","SysvarC1ock11111111111111111111111111111111","Vote111111111111111111111111111111111111111"],"header":{"numReadonlySignedAccounts":0,"numReadonlyUnsignedAccounts":3,"numRequiredSignatures":1},"instructions":[{"accounts":[1,2,3,0],"data":"37u9WtQpcm6ULa3VpnRF6CqaEftTmTtPNaoq5eaAxgTubYiNR7Jf3wpSvqhp2P7c7Dx68kAb","programIdIndex":4,"stackHeight":null}],"recentBlockhash":"8sxZZjKCz85m5ZsgPzhE1z5tSqzjcsXM45M78sPb55ET"},"signatures":["3rT1ykzcVCnGX3M3bauJ2i7eLBemrfvV2ZsigGSxpP3dE2iPUtaqUNGprfRP5PstEXPn5M2JX12m8oJXrRVUXCEb"]}}]},"id":0}"#;
    const EXPECTED_BLOCKHASH: &str = "FNy3uy9b9EMupvMpzG6Waqtbpt5Hto3naW2NnDwL1eYq";

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
            "sol_getBlock",
            encode_args((
                MAINNET_PROVIDER_ID,
                SLOT,
                TransactionDetails::Signatures,
                MAX_RESPONSE_BYTES,
            ))
            .unwrap(),
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<TaggedUiConfirmedBlock>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    assert_eq!(result.unwrap().blockhash, EXPECTED_BLOCKHASH);
}

#[tokio::test]
async fn test_get_blocks_mock() {
    const START_SLOT: u64 = 5_000_000;
    const LAST_SLOT: u64 = 5_000_010;

    const RESPONSE: &[u8] = br#"{"jsonrpc":"2.0","result":[5000000,5000001,5000002,5000003,5000004,5000005,5000006,5000007,5000008,5000009,5000010],"id":0}"#;

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
            "sol_getBlocks",
            encode_args((MAINNET_PROVIDER_ID, START_SLOT, Some(LAST_SLOT))).unwrap(),
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<Vec<u64>>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    assert_eq!(
        result.unwrap(),
        vec![5000000, 5000001, 5000002, 5000003, 5000004, 5000005, 5000006, 5000007, 5000008, 5000009, 5000010,]
    );
}

#[tokio::test]
async fn test_get_token_accounts_by_delegate_mock() {
    const PUBKEY: &str = "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T";
    const PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
    const RESPONSE: &[u8] = br#"{"jsonrpc":"2.0","result":{"context":{"slot":1114},"value":[{"account":{"data":{"program":"spl-token","parsed":{"info":{"tokenAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},"delegate":"4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T","delegatedAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},"state":"initialized","isNative":false,"mint":"3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E","owner":"CnPoSPKXu7wJqxe59Fs72tkBeALovhsCxYeFwPCQH9TD"},"type":"account"},"space":165},"executable":false,"lamports":1726080,"owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","rentEpoch":4,"space":165},"pubkey":"28YTZEwqtMHWrhWcvv34se7pjS7wctgqzCPB3gReCFKp"}]},"id":1}"#;

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
            "sol_getTokenAccountsByDelegate",
            encode_args((
                MAINNET_PROVIDER_ID,
                PUBKEY,
                TokenAccountsFilter::ProgramId(PROGRAM_ID.to_string()),
            ))
            .unwrap(),
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<Vec<TaggedRpcKeyedAccount>>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let result = result.unwrap();
    let first = result.first().unwrap();

    assert_eq!(first.pubkey, "28YTZEwqtMHWrhWcvv34se7pjS7wctgqzCPB3gReCFKp");
    assert_eq!(first.account.executable, false);
    assert_eq!(first.account.lamports, 1726080);
    assert_eq!(first.account.owner, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    assert_eq!(first.account.rent_epoch, 4);
    assert_eq!(first.account.space, Some(165));
}

#[tokio::test]
async fn test_get_token_accounts_by_owner_mock() {
    const PUBKEY: &str = "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T";
    const PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
    const RESPONSE: &[u8] = br#"{"jsonrpc":"2.0","result":{"context":{"slot":1114},"value":[{"account":{"data":{"program":"spl-token","parsed":{"info":{"tokenAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},"delegate":"4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T","delegatedAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},"state":"initialized","isNative":false,"mint":"3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E","owner":"CnPoSPKXu7wJqxe59Fs72tkBeALovhsCxYeFwPCQH9TD"},"type":"account"},"space":165},"executable":false,"lamports":1726080,"owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","rentEpoch":4,"space":165},"pubkey":"28YTZEwqtMHWrhWcvv34se7pjS7wctgqzCPB3gReCFKp"}]},"id":1}"#;

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
            "sol_getTokenAccountsByOwner",
            encode_args((
                MAINNET_PROVIDER_ID,
                PUBKEY,
                TokenAccountsFilter::ProgramId(PROGRAM_ID.to_string()),
            ))
            .unwrap(),
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<Vec<TaggedRpcKeyedAccount>>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let result = result.unwrap();
    let first = result.first().unwrap();

    assert_eq!(first.pubkey, "28YTZEwqtMHWrhWcvv34se7pjS7wctgqzCPB3gReCFKp");
    assert_eq!(first.account.executable, false);
    assert_eq!(first.account.lamports, 1726080);
    assert_eq!(first.account.owner, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    assert_eq!(first.account.rent_epoch, 4);
    assert_eq!(first.account.space, Some(165));
}

#[tokio::test]
async fn test_get_token_supply_mock() {
    const MINT: &str = "TVZMHMzujTLJzDJ3Y8HXo33wUVU67sPS26eRuYNninj";
    const RESPONSE: &[u8] = br#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"1.18.23","slot":291507458},"value":{"amount":"899966960617203","decimals":6,"uiAmount":899966960.617203,"uiAmountString":"899966960.617203"}},"id":0}"#;

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
            "sol_getTokenSupply",
            encode_args((MAINNET_PROVIDER_ID, MINT)).unwrap(),
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<UiTokenAmount>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let result = result.unwrap();

    assert_eq!(result.amount, "899966960617203");
    assert_eq!(result.decimals, 6);
    assert_eq!(result.ui_amount, Some(899966960.617203));
    assert_eq!(result.ui_amount_string, "899966960.617203".to_string());
}

#[tokio::test]
async fn test_get_token_largest_accounts_mock() {
    const MINT: &str = "So11111111111111111111111111111111111111112";
    const RESPONSE: &[u8] = br#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"1.18.23","slot":291508155},"value":[{"address":"BUvduFTd2sWFagCunBPLupG8fBTJqweLw9DuhruNFSCm","amount":"2291416519000987","decimals":9,"uiAmount":2291416.519000987,"uiAmountString":"2291416.519000987"},{"address":"2nQNF8F9LLWMqdjymiLK2u8HoHMvYa4orCXsp3w65fQ2","amount":"712421225401090","decimals":9,"uiAmount":712421.22540109,"uiAmountString":"712421.22540109"},{"address":"GafNuUXj9rxGLn4y79dPu6MHSuPWeJR6UtTWuexpGh3U","amount":"369775973393568","decimals":9,"uiAmount":369775.973393568,"uiAmountString":"369775.973393568"},{"address":"8UviNr47S8eL6J3WfDxMRa3hvLta1VDJwNWqsDgtN3Cv","amount":"346523421643623","decimals":9,"uiAmount":346523.421643623,"uiAmountString":"346523.421643623"},{"address":"2eicbpitfJXDwqCuFAmPgDP7t2oUotnAzbGzRKLMgSLe","amount":"273088293895558","decimals":9,"uiAmount":273088.293895558,"uiAmountString":"273088.293895558"},{"address":"DfYCNezifxAEsQbAJ1b3j6PX3JVBe8fu11KBhxsbw5d2","amount":"251047800078847","decimals":9,"uiAmount":251047.800078847,"uiAmountString":"251047.800078847"},{"address":"5Zumc1SYPmQ89nqwXqzogeuhdJ85iEMpSk35A4P87pmD","amount":"189657851688438","decimals":9,"uiAmount":189657.851688438,"uiAmountString":"189657.851688438"},{"address":"GuXKCb9ibwSeRSdSYqaCL3dcxBZ7jJcj6Y7rDwzmUBu9","amount":"150148879844273","decimals":9,"uiAmount":150148.879844273,"uiAmountString":"150148.879844273"},{"address":"7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5","amount":"143008982456158","decimals":9,"uiAmount":143008.982456158,"uiAmountString":"143008.982456158"},{"address":"F7tcS67EfP4bBJhWLxCk6ZmPVcsmPnJvPLQcDw5eeR67","amount":"123626802779403","decimals":9,"uiAmount":123626.802779403,"uiAmountString":"123626.802779403"},{"address":"BhNdEGJef9jSqT1iCEkFZ2bYZCdpC1vuiWtqDt87vBVp","amount":"111265135102712","decimals":9,"uiAmount":111265.135102712,"uiAmountString":"111265.135102712"},{"address":"7dSiEK9yWTxxSWpMkjHpY968nJ4Xj4aNgK3sgM23nCeL","amount":"103700132432281","decimals":9,"uiAmount":103700.132432281,"uiAmountString":"103700.132432281"},{"address":"HiLcngHP5y1Jno53tuuNeFHKWhyyZp3XuxtKPszD6rG2","amount":"92845304591505","decimals":9,"uiAmount":92845.304591505,"uiAmountString":"92845.304591505"},{"address":"9Hst4fTfQJXp1fxyVx1Lk1TubjNegFwXCedZkMRPaYAK","amount":"88176311056277","decimals":9,"uiAmount":88176.311056277,"uiAmountString":"88176.311056277"},{"address":"EUuUbDcafPrmVTD5M6qoJAoyyNbihBhugADAxRMn5he9","amount":"69211352603916","decimals":9,"uiAmount":69211.352603916,"uiAmountString":"69211.352603916"},{"address":"HZeLxbZ9uHtSpwZC3LBr4Nubd14iHwz7bRSghRZf5VCG","amount":"66669857902120","decimals":9,"uiAmount":66669.85790212,"uiAmountString":"66669.85790212"},{"address":"7e9ExBAvDvuJP3GE6eKL5aSMi4RfXv3LkQaiNZBPmffR","amount":"53644585856285","decimals":9,"uiAmount":53644.585856285,"uiAmountString":"53644.585856285"},{"address":"2hNHZg7XBhuhHVZ3JDEi4buq2fPQwuWBdQ9xkH7t1GQX","amount":"53552534347763","decimals":9,"uiAmount":53552.534347763,"uiAmountString":"53552.534347763"},{"address":"n6CwMY77wdEftf2VF6uPvbusYoraYUci3nYBPqH1DJ5","amount":"45389202140256","decimals":9,"uiAmount":45389.202140256,"uiAmountString":"45389.202140256"},{"address":"2kjCeDKKK9pCiDqfsbS72q81RZiUnSwoaruuwz1avUWn","amount":"40541538030945","decimals":9,"uiAmount":40541.538030945,"uiAmountString":"40541.538030945"}]},"id":0}"#;

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
            "sol_getTokenLargestAccounts",
            encode_args((MAINNET_PROVIDER_ID, MINT)).unwrap(),
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<Vec<TaggedRpcTokenAccountBalance>>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let result = result.unwrap();

    assert_eq!(result.len(), 20);

    let first = result.first().unwrap();

    assert_eq!(first.address, "BUvduFTd2sWFagCunBPLupG8fBTJqweLw9DuhruNFSCm");
    assert_eq!(first.amount.amount, "2291416519000987");
    assert_eq!(first.amount.decimals, 9);
    assert_eq!(first.amount.ui_amount, Some(2291416.519000987));
    assert_eq!(first.amount.ui_amount_string, "2291416.519000987".to_string());
}

#[tokio::test]
async fn test_get_block_height_mock() {
    const RESPONSE: &[u8] = br#"{"jsonrpc":"2.0","result":269611304,"id":0}"#;
    const EXPECTED: u64 = 269611304;

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
            "sol_getBlockHeight",
            encode_args((MAINNET_PROVIDER_ID,)).unwrap(),
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
        additional_responses: vec![],
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
        additional_responses: vec![],
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
        additional_responses: vec![],
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
            0x01, 0x00, 0x00, 0x00, 0x05, 0xea, 0x9c, 0xf1, 0x6c, 0xe4, 0x11, 0x98, 0xf1, 0xa4, 0x99, 0x37, 0xc8, 0x8c,
            0x37, 0x0a, 0x94, 0xd4, 0xaf, 0xff, 0x89, 0xb5, 0xba, 0xcb, 0x8e, 0xf4, 0x5e, 0x63, 0x24, 0xbb, 0x78, 0xf7,
            0x7a, 0x9f, 0x13, 0x85, 0xe3, 0xb6, 0x06, 0x00, 0x06, 0x01, 0x01, 0x00, 0x00, 0x00, 0x05, 0xea, 0x9c, 0xf1,
            0x6c, 0xe4, 0x11, 0x98, 0xf1, 0xa4, 0x99, 0x37, 0xc8, 0x8c, 0x37, 0x0a, 0x94, 0xd4, 0xaf, 0xff, 0x89, 0xb5,
            0xba, 0xcb, 0x8e, 0xf4, 0x5e, 0x63, 0x24, 0xbb, 0x78, 0xf7,
        ],
        owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap(),
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<Option<Account>>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let account = result.expect("RPC error occurred");

    assert_eq!(account, expected);
}

#[tokio::test]
async fn test_get_transaction_mock() {
    const SIGNATURE: &str = "3kxL8Qvp16kmVNiUkQSJ3zLvCJDK4qPZZ1ZL8W2VHeYoJUJnQ4VqMHFMNSmsGBq7rTfpe8cTzCopMSNRen6vGFt1";

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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    let (result,): (ic_solana::rpc_client::RpcResult<TaggedEncodedConfirmedTransactionWithStatusMeta>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let tx = result.expect("RPC error occurred");

    assert_eq!(tx, expected);
}

#[tokio::test]
async fn test_send_raw_transaction_mock() {
    const SEND_RAW_TRANSACTION_REQUEST: &str = r###"5Pqoict6REKLCaPAWadjLm2Vj5JKwvEVwJFfxYP8JiuC8FDUdxVhWUebF4gEcCR5mao8WizwGEKYRTC5X47TEN4HqUqJjcqSx1FMtyjvF37rXGqqUWm69GwxCTuzuEEFvZcguoBLemroUSmZimkPiE33J7i6RpRmZzLNJ2owzsSYHKZr5NHXaBN5hHiyzuDAakVqK1VRdyTKsVrA53xuLom7uHb7oMvYKPPWpuEaMxigWHcjfq3782no5gK1csSMEdaQavh4vU6CCr4kroM4X5KHd5qmz3TXGEEum"###;
    const SEND_RAW_TRANSACTION_RESPONSE: &[u8] = br###"{"jsonrpc":"2.0","result":"418jqKR8c5pFDNo7h9vdyFaCdhQKr9z3rmEvkaz8Fcd34eSsAhsvsHkpffYmu5YfsvHWu1uFGo48Dbm3no8cFobM","id":0}"###;
    const EXPECTED: &str = "418jqKR8c5pFDNo7h9vdyFaCdhQKr9z3rmEvkaz8Fcd34eSsAhsvsHkpffYmu5YfsvHWu1uFGo48Dbm3no8cFobM";

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
        additional_responses: vec![],
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
    const EXPECTED: &str = "2EanSSkn5cjv9DVKik5gtBkN1wwbV1TAXQQ5yu2RTPGwgrhEywVAQR2veu895uCDzvYwWZe6vD1Bcn8s7r22W17w";

    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_ii_subnet()
        .with_nns_subnet()
        .build_async()
        .await;

    let canister_id = init(&pic).await;

    let call_result = pic
        .update_call(canister_id, *USER_PRINCIPAL, "sol_address", encode_one(()).unwrap())
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
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    fast_forward(&pic, 10).await;

    let reqs = pic.get_canister_http().await;
    let req = reqs.first().unwrap();

    let mock = MockCanisterHttpResponse {
        subnet_id: req.subnet_id,
        request_id: req.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: SEND_TRANSACTION_RESPONSE.to_vec(),
        }),
        additional_responses: vec![],
    };

    pic.mock_canister_http_response(mock).await;

    fast_forward(&pic, 4).await;

    let (result,): (ic_solana::rpc_client::RpcResult<String>,) =
        decode_raw_wasm_result(&pic.await_call(call).await.unwrap()).unwrap();

    let signature = result.expect("RPC error occurred");

    assert_eq!(signature, EXPECTED.to_string());
}

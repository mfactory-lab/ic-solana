mod setup;

use std::collections::HashMap;

use ic_solana::{
    metrics::{MetricRpcHost, Metrics},
    request::RpcRequest,
    rpc_client::{RpcConfig, RpcResult, RpcServices},
    types::{
        tagged::{
            EncodedConfirmedTransactionWithStatusMeta, RpcKeyedAccount, RpcSimulateTransactionResult,
            RpcTokenAccountBalance, UiAccount, UiConfirmedBlock,
        },
        Cluster, EpochInfo, EpochSchedule, RpcAccountBalance, RpcAccountInfoConfig, RpcBlockCommitment, RpcBlockConfig,
        RpcBlockProduction, RpcBlockhash, RpcConfirmedTransactionStatusWithSignature, RpcContactInfo, RpcIdentity,
        RpcInflationGovernor, RpcInflationRate, RpcInflationReward, RpcLargestAccountsConfig, RpcLargestAccountsFilter,
        RpcLeaderSchedule, RpcPerfSample, RpcPrioritizationFee, RpcSignatureStatusConfig, RpcSimulateTransactionConfig,
        RpcSnapshotSlotInfo, RpcSupply, RpcTokenAccountsFilter, RpcVersionInfo, RpcVoteAccountStatus,
        TransactionDetails, TransactionStatus, UiDataSliceConfig, UiTokenAmount, UiTransactionEncoding,
    },
};
use ic_solana_rpc::{auth::Auth, state::InitArgs, types::RegisterProviderArgs};
use test_utils::{MockOutcallBuilder, TestSetup};

use crate::setup::{mock_update, SolanaRpcSetup, MOCK_RAW_TX};

#[test]
fn should_canonicalize_json_response() {
    let setup = SolanaRpcSetup::default();
    let responses = [
        r#"{"id":1,"jsonrpc":"2.0","result":"ok"}"#,
        r#"{"result":"ok","id":1,"jsonrpc":"2.0"}"#,
        r#"{"result":"ok","jsonrpc":"2.0","id":1}"#,
    ]
    .iter()
    .map(|&response| {
        setup
            .request(RpcServices::Mainnet, "getHealth", "", 1000)
            .mock_http(MockOutcallBuilder::new(200, response))
            .wait()
    })
    .collect::<Vec<_>>();
    assert!(responses.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn test_get_account_info() {
    let res = mock_update::<_, Option<UiAccount>>(
        "sol_getAccountInfo",
        (RpcServices::Mainnet, (), "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
        r#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"2.0.14","slot":336234816},"value":{"data":["","base58"],"executable":false,"lamports":88847814690250,"owner":"11111111111111111111111111111111","rentEpoch":18446744073709551615,"space":0}},"id":1}"#,
    )
        .unwrap()
        .unwrap();
    assert_eq!(res.owner, "11111111111111111111111111111111");
}

#[test]
fn test_get_balance() {
    let res = mock_update::<_, u64>(
        "sol_getBalance",
        (RpcServices::Mainnet, (), "AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY"),
        r#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"2.0.8","slot":324152736},"value":228},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 228);
}

#[test]
fn test_get_block() {
    let conf = RpcBlockConfig {
        transaction_details: Some(TransactionDetails::Signatures),
        ..RpcBlockConfig::default()
    };

    let res = mock_update::<_, UiConfirmedBlock>(
        "sol_getBlock",
        (RpcServices::Mainnet, (), 123u64, conf),
        r#"{"jsonrpc":"2.0","result":{"blockHeight":278596380,"blockTime":1712339598,"blockhash":"DuDqV3wED1kmx8RarqTmzGbwjCdBCyswMu6vSuAJGmhQ","parentSlot":290304299,"previousBlockhash":"2MssJkEXpFZthuFFUMuy9ZKtPXvg4TnUWog1E3vq2QPd","signatures":["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"]},"id":1}"#,
    ).unwrap();
    assert_eq!(res.blockhash, "DuDqV3wED1kmx8RarqTmzGbwjCdBCyswMu6vSuAJGmhQ");

    assert!(mock_update::<_, UiConfirmedBlock>(
        "sol_getBlock",
        (RpcServices::Mainnet, (), 123u64, conf),
        r#"{"jsonrpc":"2.0","error":{"code":-32009,"message":"Slot 500000 was skipped, or missing in long-term storage"},"id":1}"#,
    ).is_err());
}

#[test]
fn test_get_block_commitment() {
    let res = mock_update::<_, RpcBlockCommitment>(
        "sol_getBlockCommitment",
        (RpcServices::Mainnet, (), 5u64),
        r#"{"jsonrpc":"2.0","result":{"commitment":null,"totalStake":158701670213924085},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.total_stake, 158701670213924085);
}

#[test]
fn test_get_block_height() {
    let res = mock_update::<_, u64>(
        "sol_getBlockHeight",
        (RpcServices::Mainnet,),
        r#"{"jsonrpc":"2.0","result":1234,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 1234);
}

#[test]
fn test_get_block_production() {
    let res = mock_update::<_, RpcBlockProduction>(
        "sol_getBlockProduction",
        (RpcServices::Mainnet,),
        r#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"2.0.14","slot":336245863},"value":{"byIdentity":{"5DY4ft6omjdCkbdgwLKtWdLvfS1QTtETuW1i2YQ6gn9E":[4,0],"88B9r8s7adZZirAxGNQfvT2Zy7vwZtfCATaX27ZLP7Y1":[36,34],"8UN4taV45cM3EXgBxBntdTcgccHA2b4ByBHTe2FfmMvo":[4,0],"97YUjL2EK42M6jG5VA4fKuVxGXDfxsC5Zawd9haLQJGk":[1820,1615],"BrX9Z85BbmXYMjvvuAWU8imwsAqutVQiDg9uNfTGkzrJ":[1956,1956],"Cw6X5R68muAyGRCb7W8ZSP2YbaRjwMs1t5sBEPkhdwbM":[2216,2216],"FDQHfbqgSUk94XKFKWu6E8qidL7bwGEXDPzAoTVTXEDm":[16,0],"HMU77m6WSL9Xew9YvVCgz1hLuhzamz74eD9avi4XPdr":[156,0],"HPpYXZ944SXpJB3Tb7Zzy2K7YD45zGREsGqPtEP43xBx":[12,0],"dv1ZAGvdsz5hHLwWXsVnM94hWf1pjbKVau1QVkaMJ92":[36096,36077],"dv2eQHeP4RFrJZ6UeiZWoc3XTtmtZCUKxxCApCDcRNV":[35904,35894],"dv3qDFk1DTF36Z62bNvrCXe9sKATA6xvVy6A798xxAS":[35516,35505],"dv4ACNkpYPcE3aKmYDqZm9G5EB3J4MRoeE7WNDRBVJB":[36128,36127]},"range":{"firstSlot":336096000,"lastSlot":336245863}}},"id":1}"#
    ).unwrap();
    assert_eq!(res.range.first_slot, 336096000);
    assert_eq!(res.range.last_slot, Some(336245863));
}

#[test]
fn test_get_block_time() {
    let res = mock_update::<_, i64>(
        "sol_getBlockTime",
        (RpcServices::Mainnet, (), 290304000u64),
        r#"{"jsonrpc":"2.0","result":1712339486,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 1712339486);
}

#[test]
fn test_get_blocks() {
    let res = mock_update::<_, Vec<u64>>(
        "sol_getBlocks",
        (RpcServices::Mainnet, (), 290304000u64, 290304010u64),
        r#"{"jsonrpc":"2.0","result":[290304000,290304001,290304002,290304003,290304004,290304005,290304006,290304007,290304008,290304009,290304010],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res[0], 290304000);
    assert_eq!(res[10], 290304010);
}

#[test]
fn test_get_blocks_with_limit() {
    let res = mock_update::<_, Vec<u64>>(
        "sol_getBlocksWithLimit",
        (RpcServices::Mainnet, (), 290304000u64, 3u64),
        r#"{"jsonrpc":"2.0","result":[290304000,290304001,290304002],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res[0], 290304000);
    assert_eq!(res[2], 290304002);
}

#[test]
fn test_get_cluster_nodes() {
    let res = mock_update::<_, Vec<RpcContactInfo>>(
        "sol_getClusterNodes",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc": "2.0","result":[{"gossip":"10.239.6.48:8001","pubkey":"9QzsJf7LPLj8GkXbYT3LFDKqsj2hHG7TA3xinJHu8epQ","rpc":"10.239.6.48:8899","tpu":"10.239.6.48:8856","version":"1.0.0 c375ce1f"}],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res[0].pubkey, "9QzsJf7LPLj8GkXbYT3LFDKqsj2hHG7TA3xinJHu8epQ");
}

#[test]
fn test_get_epoch_info() {
    let res = mock_update::<_, EpochInfo>(
        "sol_getEpochInfo",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"absoluteSlot":336248665,"blockHeight":324449642,"epoch":778,"slotIndex":152665,"slotsInEpoch":432000,"transactionCount":14602602834},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.epoch, 778);
}

#[test]
fn test_get_epoch_schedule() {
    let res = mock_update::<_, EpochSchedule>(
        "sol_getEpochSchedule",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"firstNormalEpoch":0,"firstNormalSlot":0,"leaderScheduleSlotOffset":432000,"slotsPerEpoch":4320000,"warmup":false},"id":1}"#,
    )
        .unwrap();
    assert_eq!(res.slots_per_epoch, 4320000);
}

#[test]
fn test_get_fee_for_message() {
    let res = mock_update::<_, u64>(
        "sol_getFeeForMessage",
        (RpcServices::Mainnet, (), "AQABAgIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEBAQAA"),
        r#"{"jsonrpc": "2.0","result": { "context": { "slot": 5068 }, "value": 5000 },"id": 1}"#,
    )
    .unwrap();
    assert_eq!(res, 5000);
}

#[test]
fn test_get_first_available_block() {
    let res = mock_update::<_, u64>(
        "sol_getFirstAvailableBlock",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":1,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 1);
}

#[test]
fn test_get_genesis_hash() {
    let res = mock_update::<_, String>(
        "sol_getGenesisHash",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":"GH7ome3EiwEr7tu9JuTh2dpYWBJK3z69Xm1ZE3MEE6JC","id":1}"#,
    )
    .unwrap();
    assert_eq!(res, "GH7ome3EiwEr7tu9JuTh2dpYWBJK3z69Xm1ZE3MEE6JC");
}

#[test]
fn test_get_health() {
    assert_eq!(
        mock_update::<_, String>(
            "sol_getHealth",
            (RpcServices::Mainnet, ()),
            r#"{"jsonrpc":"2.0","result":"ok","id":1}"#,
        )
        .unwrap(),
        "ok"
    );

    assert!(mock_update::<_, String>(
        "sol_getHealth",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","error":{"code":-32005,"message":"Node is unhealthy","data":{}},"id":1}"#,
    )
    .is_err());
}

#[test]
fn test_get_highest_snapshot_slot() {
    let res = mock_update::<_, RpcSnapshotSlotInfo>(
        "sol_getHighestSnapshotSlot",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"full":336249023,"incremental":336251523},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.full, 336249023);
}

#[test]
fn test_get_identity() {
    let res = mock_update::<_, RpcIdentity>(
        "sol_getIdentity",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"identity":"9QzsJf7LPLj8GkXbYT3LFDKqsj2hHG7TA3xinJHu8epQ"},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.identity, "9QzsJf7LPLj8GkXbYT3LFDKqsj2hHG7TA3xinJHu8epQ");
}

#[test]
fn test_get_inflation_governor() {
    let res = mock_update::<_, RpcInflationGovernor>(
        "sol_getInflationGovernor",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"foundation":0.0,"foundationTerm":0.0,"initial":0.08,"taper":0.15,"terminal":0.015},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.initial, 0.08);
    assert_eq!(res.taper, 0.15);
    assert_eq!(res.terminal, 0.015);
}

#[test]
fn test_get_inflation_rate() {
    let res = mock_update::<_, RpcInflationRate>(
        "sol_getInflationRate",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"epoch":778,"foundation":0.0,"total":0.041593020358344515,"validator":0.041593020358344515},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.epoch, 778);
}

#[test]
fn test_get_inflation_reward() {
    let res = mock_update::<_, Vec<Option<RpcInflationReward>>>(
        "sol_getInflationReward",
        (RpcServices::Mainnet, (), vec!["6dmNQ5jwLeLk5REvio1JcMshcbvkYMwy26sJ8pbkvStu", "BGsqMegLpV6n6Ve146sSX2dTjUMj3M92HnU8BbNRMhF2"]),
        r#"{"jsonrpc":"2.0","result":[{"amount":2500,"effectiveSlot":224,"epoch":2,"postBalance":499999442500},null],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res[0].as_ref().unwrap().amount, 2500);
    assert!(res[1].as_ref().is_none());
}

#[test]
fn test_get_signatures_for_address() {
    let res = mock_update::<_, Vec<RpcConfirmedTransactionStatusWithSignature>>(
        "sol_getSignaturesForAddress",
        (RpcServices::Mainnet, (), "Vote111111111111111111111111111111111111111"),
        r#"{"jsonrpc":"2.0","result":[{"blockTime":1730179716,"confirmationStatus":"finalized","err":null,"memo":null,"signature":"255gS6xy2wZkW1RgQybsvoc91LTVC6C1HtPy3o6wqc9m4UhPf47DtnJYUaKD9MvxRWtfy246fpAWEWqyvwQDLpLE","slot":336253303}],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res[0].slot, 336253303);
}

#[test]
fn test_get_slot() {
    let res = mock_update::<_, u64>(
        "sol_getSlot",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":336253303,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 336253303);
}

#[test]
fn test_get_slot_leader() {
    let res = mock_update::<_, String>(
        "sol_getSlotLeader",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":"Hb9LRL5HsXKdLXZt6Uj6XyHnJtX8YiJLwCkK5HfQ5yU","id":1}"#,
    )
    .unwrap();
    assert_eq!(res, "Hb9LRL5HsXKdLXZt6Uj6XyHnJtX8YiJLwCkK5HfQ5yU");
}

#[test]
fn test_get_slot_leaders() {
    let res = mock_update::<_, Vec<String>>(
        "sol_getSlotLeaders",
        (RpcServices::Mainnet, (), 10u64),
        r#"{"jsonrpc":"2.0","result":["dv3qDFk1DTF36Z62bNvrCXe9sKATA6xvVy6A798xxAS","dv3qDFk1DTF36Z62bNvrCXe9sKATA6xvVy6A798xxAS"],"id":1}"#,
    )
    .unwrap();

    assert_eq!(
        res,
        vec![
            "dv3qDFk1DTF36Z62bNvrCXe9sKATA6xvVy6A798xxAS",
            "dv3qDFk1DTF36Z62bNvrCXe9sKATA6xvVy6A798xxAS"
        ]
    );
}

#[test]
fn test_get_stake_minimum_delegation() {
    let res = mock_update::<_, u64>(
        "sol_getStakeMinimumDelegation",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"context":{"slot":501},"value":1000000000},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 1000000000);
}

#[test]
fn test_get_supply() {
    let res = mock_update::<_, RpcSupply>(
        "sol_getSupply",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"2.0.14","slot":336257703},"value":{"circulating":943993901895912059,"nonCirculating":315864941241040354,"nonCirculatingAccounts":["CWeRmXme7LmbaUWTZWFLt6FMnpzLCHaQLuR2TdgFn4Lq"],"total":1259858843136952413}},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.circulating, 943993901895912059);
    assert_eq!(res.non_circulating, 315864941241040354);
    assert_eq!(res.total, 1259858843136952413);
}

#[test]
fn test_get_token_supply() {
    let res = mock_update::<_, UiTokenAmount>(
        "sol_getTokenSupply",
        (RpcServices::Mainnet, (), "3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E"),
        r#"{"jsonrpc":"2.0","result":{"context":{"slot":1114},"value":{"amount":"100000","decimals":2,"uiAmount":1000,"uiAmountString":"1000"}},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.amount, "100000");
}

#[test]
fn test_get_token_account_balance() {
    let res = mock_update::<_, UiTokenAmount>(
        "sol_getTokenAccountBalance",
        (RpcServices::Mainnet, (), "2mMRrstGWsueujXeQnUUgp3VZBhJt8FWxroYDec6eCUw"),
        r#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"1.18.25","slot":298362228},"value":{"amount":"0","decimals":6,"uiAmount":0.0,"uiAmountString":"0"}},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.amount, "0");
}

#[test]
fn test_get_token_accounts_by_delegate() {
    let res = mock_update::<_, Vec<RpcKeyedAccount>>(
        "sol_getTokenAccountsByDelegate",
        (RpcServices::Mainnet, (), "2mMRrstGWsueujXeQnUUgp3VZBhJt8FWxroYDec6eCUw", RpcTokenAccountsFilter::Mint("3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E".into())),
        r#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"1.18.25","slot":298362228},"value":[{"account":{"data":{"program":"spl-token","parsed":{"info":{"tokenAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},"delegate":"4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T","delegatedAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},"state":"initialized","isNative":false,"mint":"3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E","owner":"CnPoSPKXu7wJqxe59Fs72tkBeALovhsCxYeFwPCQH9TD"},"type":"account"},"space":165},"executable":false,"lamports":1726080,"owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","rentEpoch":4,"space":165},"pubkey":"28YTZEwqtMHWrhWcvv34se7pjS7wctgqzCPB3gReCFKp"}]},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn test_get_token_accounts_by_owner() {
    let res = mock_update::<_, Vec<RpcKeyedAccount>>(
        "sol_getTokenAccountsByOwner",
        (RpcServices::Mainnet, (), "2mMRrstGWsueujXeQnUUgp3VZBhJt8FWxroYDec6eCUw", RpcTokenAccountsFilter::Mint("3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E".into())),
         r#"{"jsonrpc":"2.0","result":{"context":{"slot":1114},"value":[{"account":{"data":{"program":"spl-token","parsed":{"info":{"tokenAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},"delegate":"4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T","delegatedAmount":{"amount":"1","decimals":1,"uiAmount":0.1,"uiAmountString":"0.1"},"state":"initialized","isNative":false,"mint":"3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E","owner":"CnPoSPKXu7wJqxe59Fs72tkBeALovhsCxYeFwPCQH9TD"},"type":"account"},"space":165},"executable":false,"lamports":1726080,"owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","rentEpoch":4,"space":165},"pubkey":"28YTZEwqtMHWrhWcvv34se7pjS7wctgqzCPB3gReCFKp"}]},"id":1}"#
    )
    .unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn test_token_largest_accounts() {
    let res = mock_update::<_, Vec<RpcTokenAccountBalance>>(
        "sol_getTokenLargestAccounts",
        (RpcServices::Mainnet, (), "3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E", RpcLargestAccountsConfig { commitment: None, filter: Some(RpcLargestAccountsFilter::NonCirculating) }),
        r#"{"jsonrpc":"2.0","result":{"context":{"slot":1114},"value":[{"address":"FYjHNoFtSQ5uijKrZFyYAxvEr87hsKXkXcxkcmkBAf4r","amount":"771","decimals":2,"uiAmount":7.71,"uiAmountString":"7.71"},{"address":"BnsywxTcaYeNUtzrPxQUvzAWxfzZe3ZLUJ4wMMuLESnu","amount":"229","decimals":2,"uiAmount":2.29,"uiAmountString":"2.29"}]},"id":1}"#,
    )
        .unwrap();
    assert_eq!(res.len(), 2);
}

#[test]
fn test_get_largest_accounts() {
    let res = mock_update::<_, Vec<RpcAccountBalance>>(
        "sol_getLargestAccounts",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":[{"lamports":1726080,"address":"28YTZEwqtMHWrhWcvv34se7pjS7wctgqzCPB3gReCFKp"}],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn test_get_latest_blockhash() {
    let res = mock_update::<_, RpcBlockhash>(
        "sol_getLatestBlockhash",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"context":{"slot":2792},"value":{"blockhash":"EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N","lastValidBlockHeight":3090}},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.blockhash, "EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N");
}

#[test]
fn test_get_leader_schedule() {
    let res = mock_update::<_, RpcLeaderSchedule>(
        "sol_getLeaderSchedule",
        (RpcServices::Mainnet, (), 700u64),
        r#"{"jsonrpc":"2.0","result":{"4Qkev8aNZcqFNSRhQzwyLMFSsi94jHqE8WNVTJzTP99F":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63]},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn test_get_max_retransmit_slot() {
    let res = mock_update::<_, u64>(
        "sol_getMaxRetransmitSlot",
        (RpcServices::Mainnet,),
        r#"{"jsonrpc":"2.0","result":2792,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 2792);
}

#[test]
fn test_get_max_shred_insert_slot() {
    let res = mock_update::<_, u64>(
        "sol_getMaxShredInsertSlot",
        (RpcServices::Mainnet,),
        r#"{"jsonrpc":"2.0","result":2792,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 2792);
}

#[test]
fn test_get_minimum_balance_for_rent_exemption() {
    let res = mock_update::<_, u64>(
        "sol_getMinimumBalanceForRentExemption",
        (RpcServices::Mainnet, (), 42usize),
        r#"{"jsonrpc":"2.0","result":2792,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 2792);
}

#[test]
fn test_get_multiple_accounts() {
    let res = mock_update::<_, Vec<UiAccount>>(
        "sol_getMultipleAccounts",
        (
            RpcServices::Mainnet,
            RpcConfig { response_size_estimate: Some(1024 * 1024), response_consensus: None },
            ["Fg6PaFpoGXkYsidMpWTK6W2beZ7FEfcYkg476zPFsLnS"],
            RpcAccountInfoConfig {
                encoding: None,
                data_slice: Some(UiDataSliceConfig { offset: 10, length: 100 }),
                commitment: None,
                min_context_slot: None,
            }
        ),
        r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":[{"data":["","base64"],"executable":false,"lamports":1000000000,"owner":"11111111111111111111111111111111","rentEpoch":2,"space":16},{"data":["","base64"],"executable":false,"lamports":5000000000,"owner":"11111111111111111111111111111111","rentEpoch":2,"space":0}]},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.len(), 2);
}

#[test]
fn test_get_program_accounts() {
    let res = mock_update::<_, Vec<RpcKeyedAccount>>(
        "sol_getProgramAccounts",
        (RpcServices::Mainnet, (), "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        r#"{"jsonrpc":"2.0","result":[{"account":{"data":"2R9jLfiAQ9bgdcw6h8s44439","executable":false,"lamports":15298080,"owner":"4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T","rentEpoch":28,"space":42},"pubkey":"CxELquR1gPP8wHe33gZ4QxqGB3sZ9RSwsJ2KshVewkFY"}],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.len(), 1);
}

#[test]
fn test_get_recent_performance_samples() {
    let res = mock_update::<_, Vec<RpcPerfSample>>(
        "sol_getRecentPerformanceSamples",
        (RpcServices::Mainnet, (), 4u64),
        r#"{"jsonrpc":"2.0","result":[{"numSlots":126,"numTransactions":126,"numNonVoteTransactions":1,"samplePeriodSecs":60,"slot":348125},{"numSlots":126,"numTransactions":126,"numNonVoteTransactions":1,"samplePeriodSecs":60,"slot":347999},{"numSlots":125,"numTransactions":125,"numNonVoteTransactions":0,"samplePeriodSecs":60,"slot":347873},{"numSlots":125,"numTransactions":125,"numNonVoteTransactions":0,"samplePeriodSecs":60,"slot":347748}],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.len(), 4);
}

#[test]
fn test_get_recent_prioritization_fees() {
    let res = mock_update::<_, Vec<RpcPrioritizationFee>>(
        "sol_getRecentPrioritizationFees",
        (RpcServices::Mainnet, (), ["CxELquR1gPP8wHe33gZ4QxqGB3sZ9RSwsJ2KshVewkFY"]),
        r#"{"jsonrpc":"2.0","result":[{"slot":348125,"prioritizationFee":0},{"slot":348126,"prioritizationFee":1000},{"slot":348127,"prioritizationFee":500},{"slot":348128,"prioritizationFee":0},{"slot":348129,"prioritizationFee":1234}],"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.len(), 5);
}

#[test]
fn test_get_signature_statuses() {
    let res = mock_update::<_, Vec<Option<TransactionStatus>>>(
        "sol_getSignatureStatuses",
        (RpcServices::Mainnet, (), ["5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW"], RpcSignatureStatusConfig { search_transaction_history: true }),
        r#"{"jsonrpc":"2.0","result":{"context":{"slot":82},"value":[{"slot":48,"confirmations":null,"err":null,"status":{"Ok":null},"confirmationStatus":"finalized"},null]},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.len(), 2);
}

#[test]
fn test_get_transaction() {
    let res = mock_update::<_, Option<EncodedConfirmedTransactionWithStatusMeta>>(
        "sol_getTransaction",
        (
            RpcServices::Mainnet,
            (),
            "3kxL8Qvp16kmVNiUkQSJ3zLvCJDK4qPZZ1ZL8W2VHeYoJUJnQ4VqMHFMNSmsGBq7rTfpe8cTzCopMSNRen6vGFt1",
        ),
        r#"{"jsonrpc":"2.0","result":{"blockTime":1730657183,"meta":{"computeUnitsConsumed":300,"err":null,"fee":5000,"innerInstructions":[],"loadedAddresses":{"readonly":[],"writable":[]},"logMessages":["Program 11111111111111111111111111111111 invoke [1]","Program 11111111111111111111111111111111 success","Program 11111111111111111111111111111111 invoke [1]","Program 11111111111111111111111111111111 success"],"postBalances":[0,998172448,8052016972,1],"postTokenBalances":[],"preBalances":[1200000,996978448,8052015972,1],"preTokenBalances":[],"rewards":[],"status":{"Ok":null}},"slot":299317916,"transaction":{"message":{"accountKeys":["6CY6QEogNW61ZHW7Uzt9rAprt4CJsop2ZGmn8TtrjS1b","DXMU5Xgs8Wc3qUKSSWwEv4mVnf1aEZ1FHL6JSQGjgo5","GiU1BqaWstzgbmMfksRc6Lx9cW4jQmTRCteodpSJeyMi","11111111111111111111111111111111"],"header":{"numReadonlySignedAccounts":0,"numReadonlyUnsignedAccounts":1,"numRequiredSignatures":1},"instructions":[{"accounts":[0,2],"data":"3Bxs4ffTu9T19DNF","programIdIndex":3,"stackHeight":null},{"accounts":[0,1],"data":"3Bxs43a1Fa6gnJDD","programIdIndex":3,"stackHeight":null}],"recentBlockhash":"BCKZ8D38Vb8PM5E7yPSCAjct585Z4DwdvMKZNJRxZjpQ"},"signatures":["5HfJwpqxqDiNddNcCGo9ejXBcpzCGmjkYxwuuomYECYjvDWv3ZdcNevxZMMjeXpgKpwkvMw7w4A5Aabq734cjcE7"]}},"id":1}"#,
    )
    .unwrap();
    assert!(res.is_some());
}

#[test]
fn test_get_transaction_count() {
    let res = mock_update::<_, u64>(
        "sol_getTransactionCount",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":123,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 123);
}

#[test]
fn test_get_version() {
    let res = mock_update::<_, RpcVersionInfo>(
        "sol_getVersion",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","result":{"solana-core":"1.9.20"},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.solana_core, "1.9.20");
}

#[test]
fn test_vote_accounts() {
    let res = mock_update::<_, RpcVoteAccountStatus>(
        "sol_getVoteAccounts",
        (RpcServices::Mainnet,),
        r#"{"jsonrpc":"2.0","result":{"current":[{"activatedStake":38027760856115571,"commission":100,"epochCredits":[[774,504510309,497621700],[775,511402034,504510309],[776,518294287,511402034],[777,525180874,518294287],[778,529817398,525180874]],"epochVoteAccount":true,"lastVote":336387304,"nodePubkey":"dv4ACNkpYPcE3aKmYDqZm9G5EB3J4MRoeE7WNDRBVJB","rootSlot":336387273,"votePubkey":"23AoPQc3EPkfLWb14cKiWNahh1H9rtb3UBk8gWseohjF"}],"delinquent":[{"activatedStake":123466157276,"commission":0,"epochCredits":[],"epochVoteAccount":true,"lastVote":0,"nodePubkey":"CW6zzaC861mTS4TsMkvwg9oeDEEDycznbUjvPeNngemG","rootSlot":0,"votePubkey":"7BN2ep6pc7g3gCMycv9yTpZfEgccHuu7EupahynpbAqM"},{"activatedStake":126546630397,"commission":1,"epochCredits":[],"epochVoteAccount":true,"lastVote":0,"nodePubkey":"DKFyjhF4z4EXBRMc2tG2qjv1bqp63s3HQabrq3KKGwrU","rootSlot":0,"votePubkey":"C1eDhA4VnCcAqFm4LoJ86TPLmCaXQ5dLbTSLh7tTNVi4"}]},"id":1}"#,
    )
    .unwrap();
    assert_eq!(res.current[0].activated_stake, 38027760856115571);
    assert_eq!(res.current[0].commission, 100);
    assert_eq!(res.delinquent[0].activated_stake, 123466157276);
    assert_eq!(res.delinquent[0].commission, 0);
}

#[test]
fn test_is_blockhash_valid() {
    let res = mock_update::<_, bool>(
        "sol_isBlockhashValid",
        (RpcServices::Mainnet, (), "J7rBdM6AecPDEZp8aPq5iPSNKVkU5Q76F3oAV4eW5wsW"),
        r#"{"jsonrpc":"2.0","result":{"context":{"slot":2483},"value":true},"id":1}"#,
    )
    .unwrap();
    assert!(res);
}

#[test]
fn test_minimum_ledger_slot() {
    let res = mock_update::<_, u64>(
        "sol_minimumLedgerSlot",
        (RpcServices::Mainnet,),
        r#"{"jsonrpc":"2.0","result":1234,"id":1}"#,
    )
    .unwrap();
    assert_eq!(res, 1234);
}

#[test]
fn test_request_airdrop() {
    let res = mock_update::<_, String>(
        "sol_requestAirdrop",
        (
            RpcServices::Mainnet,
            (),
            "83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri",
            1000000000u64,
        ),
        r#"{"jsonrpc":"2.0","result":"5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW","id":1}"#,
    )
    .unwrap();
    assert_eq!(
        res,
        "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW"
    );
}

#[test]
fn test_send_transaction() {
    let res = mock_update::<_, String>(
        "sol_sendTransaction",
        (RpcServices::Mainnet,(), MOCK_RAW_TX),
        r#"{"jsonrpc":"2.0","result":"2id3YC2jK9G5Wo2phDx4gJVAew8DcY5NAojnVuao8rkxwPYPe8cSwE5GzhEgJA2y8fVjDEo6iR6ykBvDxrTQrtpb","id":1}"#,
    )
    .unwrap();
    assert_eq!(
        res,
        "2id3YC2jK9G5Wo2phDx4gJVAew8DcY5NAojnVuao8rkxwPYPe8cSwE5GzhEgJA2y8fVjDEo6iR6ykBvDxrTQrtpb"
    );
}

#[test]
fn test_simulate_transaction() {
    let res = mock_update::<_, RpcSimulateTransactionResult>(
        "sol_simulateTransaction",
        (
            RpcServices::Mainnet,
            (),
            MOCK_RAW_TX,
            RpcSimulateTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                ..Default::default()
            }
        ),
        r#"{"jsonrpc":"2.0","result":{"context":{"slot":218},"value":{"err":null,"accounts":null,"logs":["Program 83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri invoke [1]","Program 83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri consumed 2366 of 1400000 compute units","Program return: 83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri KgAAAAAAAAA=","Program 83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri success"],"returnData":{"data":["Kg==","base64"],"programId":"83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri"},"unitsConsumed":2366}},"id":1}"#,
    )
    .unwrap();
    assert!(res.logs.is_some());
    assert!(res.return_data.is_some());
    assert_eq!(res.units_consumed, Some(2366))
}

#[test]
fn test_get_logs() {
    type Logs = HashMap<String, RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>>;

    let res = SolanaRpcSetup::default()
        .call_update::<_, RpcResult<Logs>>(
            "sol_getLogs",
            (RpcServices::Mainnet, (), "83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri")
        )
        .mock_http_once(MockOutcallBuilder::new(
            200,
            r#"{"jsonrpc":"2.0","result":[{"blockTime":1730179716,"confirmationStatus":"finalized","err":null,"memo":null,"signature":"5HfJwpqxqDiNddNcCGo9ejXBcpzCGmjkYxwuuomYECYjvDWv3ZdcNevxZMMjeXpgKpwkvMw7w4A5Aabq734cjcE7","slot":336253303}],"id":1}"#,
        ))
        .mock_http_once(MockOutcallBuilder::new(
            200,
            r#"[{"jsonrpc":"2.0","result":{"blockTime":1730657183,"meta":{"computeUnitsConsumed":300,"err":null,"fee":5000,"innerInstructions":[],"loadedAddresses":{"readonly":[],"writable":[]},"logMessages":["Program 11111111111111111111111111111111 invoke [1]","Program 11111111111111111111111111111111 success","Program 11111111111111111111111111111111 invoke [1]","Program 11111111111111111111111111111111 success"],"postBalances":[0,998172448,8052016972,1],"postTokenBalances":[],"preBalances":[1200000,996978448,8052015972,1],"preTokenBalances":[],"rewards":[],"status":{"Ok":null}},"slot":299317916,"transaction":{"message":{"accountKeys":["6CY6QEogNW61ZHW7Uzt9rAprt4CJsop2ZGmn8TtrjS1b","DXMU5Xgs8Wc3qUKSSWwEv4mVnf1aEZ1FHL6JSQGjgo5","GiU1BqaWstzgbmMfksRc6Lx9cW4jQmTRCteodpSJeyMi","11111111111111111111111111111111"],"header":{"numReadonlySignedAccounts":0,"numReadonlyUnsignedAccounts":1,"numRequiredSignatures":1},"instructions":[{"accounts":[0,2],"data":"3Bxs4ffTu9T19DNF","programIdIndex":3,"stackHeight":null},{"accounts":[0,1],"data":"3Bxs43a1Fa6gnJDD","programIdIndex":3,"stackHeight":null}],"recentBlockhash":"BCKZ8D38Vb8PM5E7yPSCAjct585Z4DwdvMKZNJRxZjpQ"},"signatures":["5HfJwpqxqDiNddNcCGo9ejXBcpzCGmjkYxwuuomYECYjvDWv3ZdcNevxZMMjeXpgKpwkvMw7w4A5Aabq734cjcE7"]},"version":"legacy"},"id":2}]"#,
        ))
        .wait()
        .unwrap();

    assert!(res["5HfJwpqxqDiNddNcCGo9ejXBcpzCGmjkYxwuuomYECYjvDWv3ZdcNevxZMMjeXpgKpwkvMw7w4A5Aabq734cjcE7"].is_ok());
}

#[test]
fn should_get_valid_request_cost() {
    assert_eq!(
        SolanaRpcSetup::new(InitArgs {
            demo: None,
            ..Default::default()
        })
        .call_query::<_, u128>("requestCost", (MOCK_RAW_TX, 1000u64)),
        321476800
    );
}

#[test]
fn should_get_nodes_in_subnet() {
    assert_eq!(SolanaRpcSetup::default().get_nodes_in_subnet(), 34);
}

#[test]
fn should_allow_manager_to_authorize_and_deauthorize_user() {
    let setup = SolanaRpcSetup::default();
    let principal = TestSetup::principal(3);

    setup
        .clone()
        .as_controller()
        .authorize(principal, Auth::RegisterProvider)
        .wait();
    let principals = setup.get_authorized(Auth::RegisterProvider);
    assert!(principals.contains(&principal));
    setup
        .clone()
        .as_controller()
        .deauthorize(principal, Auth::RegisterProvider)
        .wait();
    let principals = setup.get_authorized(Auth::RegisterProvider);
    assert!(!principals.contains(&principal));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn should_not_allow_caller_without_access_authorize_users() {
    SolanaRpcSetup::default()
        .authorize(TestSetup::principal(9), Auth::RegisterProvider)
        .wait();
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn should_not_allow_caller_without_access_deauthorize_users() {
    SolanaRpcSetup::default()
        .deauthorize(TestSetup::principal(9), Auth::RegisterProvider)
        .wait();
}

#[test]
fn should_allow_manager_to_register_and_unregister_providers() {
    let setup = SolanaRpcSetup::default();
    let provider_id = "test_mainnet1".to_string();
    setup
        .clone()
        .as_controller()
        .register_provider(RegisterProviderArgs {
            id: provider_id.clone(),
            url: Cluster::Mainnet.url().into(),
            auth: None,
        })
        .wait();
    let providers = setup.get_providers();
    assert!(providers.contains(&provider_id));
    setup.clone().as_controller().unregister_provider(&provider_id).wait();
    let providers = setup.get_providers();
    assert!(!providers.contains(&provider_id));
}

#[test]
fn should_allow_caller_with_access_register_provider() {
    let setup = SolanaRpcSetup::default();
    let principal = TestSetup::principal(3);

    setup
        .clone()
        .as_controller()
        .authorize(principal, Auth::RegisterProvider)
        .wait();

    let provider_id = "test_mainnet1".to_string();
    setup
        .clone()
        .as_caller(principal)
        .register_provider(RegisterProviderArgs {
            id: provider_id.clone(),
            url: Cluster::Mainnet.url().into(),
            auth: None,
        })
        .wait();
    let providers = setup.get_providers();
    assert!(providers.contains(&provider_id));
    setup
        .clone()
        .as_caller(principal)
        .unregister_provider(&provider_id)
        .wait();
    let providers = setup.get_providers();
    assert!(!providers.contains(&provider_id));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn should_not_allow_caller_without_access_to_register_provider() {
    SolanaRpcSetup::default()
        .register_provider(RegisterProviderArgs {
            id: "test_mainnet1".to_string(),
            url: Cluster::Mainnet.url().into(),
            auth: None,
        })
        .wait();
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn should_not_allow_caller_without_access_to_unregister_provider() {
    SolanaRpcSetup::default().unregister_provider("mainnet").wait();
}

#[test]
fn should_retrieve_logs() {
    let setup = SolanaRpcSetup::new(InitArgs {
        demo: None,
        ..Default::default()
    });
    assert_eq!(setup.http_get_logs("DEBUG"), vec![]);
    assert_eq!(setup.http_get_logs("INFO"), vec![]);

    let principal = TestSetup::principal(3);

    setup
        .clone()
        .as_controller()
        .authorize(principal, Auth::RegisterProvider)
        .wait();

    assert_eq!(setup.http_get_logs("DEBUG"), vec![]);
    assert!(setup.http_get_logs("INFO")[0].message.contains(
        format!(
            "Authorizing `{:?}` for principal: {}",
            Auth::RegisterProvider,
            principal
        )
        .as_str()
    ));
}

#[test]
fn should_recognize_rate_limit() {
    let setup = SolanaRpcSetup::default();
    let result = setup
        .request(RpcServices::Mainnet, "getHealth", "", 1000)
        .mock_http(MockOutcallBuilder::new(
            429,
            r#"{"jsonrpc":"2.0","error":{"code":429,"message":"Too many requests for a specific RPC call"},"id":1}"#,
        ))
        .wait();

    println!("{:#?}", result);

    // TODO: fix
    // assert_eq!(
    //     result,
    //     Err(RpcError::HttpOutcallError {
    //         code: 429.into(),
    //         message: "(Rate limit error message)".to_string(),
    //     })
    // );

    let rpc_method = || RpcRequest::GetHealth.into();
    let host = MetricRpcHost(Cluster::Mainnet.host_str().unwrap());

    assert_eq!(
        setup.get_metrics(),
        Metrics {
            requests: [((rpc_method(), host.clone()), 1)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
            responses: [((rpc_method(), host, 429.into()), 1)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
            ..Default::default()
        }
    );
}

#[test]
fn upgrade_should_keep_state() {
    let setup = SolanaRpcSetup::default();
    let principal = TestSetup::principal(3);

    setup
        .clone()
        .as_controller()
        .authorize(principal, Auth::RegisterProvider)
        .wait();

    let principals = setup.get_authorized(Auth::RegisterProvider);
    assert!(principals.contains(&principal));

    setup
        .clone()
        .as_controller()
        .register_provider(RegisterProviderArgs {
            id: "test_mainnet1".to_string(),
            url: Cluster::Mainnet.url().into(),
            auth: None,
        })
        .wait();

    let providers = setup.get_providers();
    assert!(providers.contains(&"test_mainnet1".to_string()));

    setup.upgrade_canister(InitArgs::default());

    let principals = setup.get_authorized(Auth::RegisterProvider);
    assert!(principals.contains(&principal));

    let providers = setup.get_providers();
    assert!(providers.contains(&"test_mainnet1".to_string()));
}

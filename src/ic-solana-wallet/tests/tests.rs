use std::str::FromStr;

use ic_solana::{rpc_client::RpcServices, types::Pubkey};
use test_utils::MockOutcallBuilder;

mod setup;

use crate::setup::SolanaWalletSetup;

#[test]
fn test_address() {
    let setup = SolanaWalletSetup::new();
    let addr = setup.call_update::<_, String>("address", ()).wait();
    assert_eq!(addr, "F57BD4FrpkM49idKyws2WBBjyR8W8dRsepLJ4EqLP3Qb");
    let addr = setup.as_controller().call_update::<_, String>("address", ()).wait();
    assert_eq!(addr, "8GU8W7fAAy2trcy36fjVJuJhEY5uA3EYTvA7jupM72wG");
}

#[test]
fn test_sign_message() {
    let setup = SolanaWalletSetup::new();
    let message = "test123";
    let address = setup.call_update::<_, String>("address", ()).wait();
    let pubkey = Pubkey::from_str(&address).unwrap();
    let signature = setup.call_update::<_, Vec<u8>>("signMessage", (message,)).wait();
    let is_valid = pubkey.verify_signature(message.as_bytes(), &signature);
    assert!(is_valid)
}

// TODO: fix
// #[test]
#[allow(dead_code)]
fn test_send_transaction() {
    let setup = SolanaWalletSetup::new();

    let raw_tx ="4hXTCkRzt9WyecNzV1XPgCDfGAZzQKNxLXgynz5QDuWWPSAZBZSHptvWRL3BjCvzUXRdKvHL2b7yGrRQcWyaqsaBCncVG7BFggS8w9snUts67BSh3EqKpXLUm5UMHfD7ZBe9GhARjbNQMLJ1QD3Spr6oMTBU6EhdB4RD8CP2xUxr2u3d6fos36PD98XS6oX8TQjLpsMwncs5DAMiD4nNnR8NBfyghGCWvCVifVwvA8B8TJxE1aiyiv2L429BCWfyzAme5sZW8rDb14NeCQHhZbtNqfXhcp2tAnaAT";

    let signature = setup
        .call_update::<_, String>("sendTransaction", (RpcServices::Mainnet, (), raw_tx))
        .mock_http_once(MockOutcallBuilder::new(200, r#"{"jsonrpc":"2.0","result":{"context":{"slot":2792},"value":{"blockhash":"EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N","lastValidBlockHeight":3090}},"id":1}"#))
        .mock_http_once(MockOutcallBuilder::new(200, r#"{"jsonrpc":"2.0","result":"2EanSSkn5cjv9DVKik5gtBkN1wwbV1TAXQQ5yu2RTPGwgrhEywVAQR2veu895uCDzvYwWZe6vD1Bcn8s7r22W17w","id":2}"#))
        .wait();

    println!("signature: {}", signature);
}

use std::{cell::RefCell, str::FromStr};

use candid::{CandidType, Principal};
use ic_solana::{
    rpc_client::RpcResult,
    types::{AccountMeta, BlockHash, Instruction, Pubkey},
};

thread_local! {
    static SOL_PROVIDER_CANISTER: RefCell<Option<Principal>>  = const { RefCell::new(None) };
}

fn sol_canister_id() -> Principal {
    SOL_PROVIDER_CANISTER.with_borrow(|canister| canister.expect("Solana provider canister not initialized"))
}

#[derive(CandidType, Debug)]
pub struct SendTransactionRequest {
    pub instructions: Vec<String>,
    pub recent_blockhash: Option<String>,
}

#[ic_cdk::update]
async fn test() {
    let sol_canister = sol_canister_id();

    // Get the solana address associated with the caller
    let response: Result<(String,), _> = ic_cdk::call(sol_canister, "sol_address", ()).await;

    // d|devnet|m|mainnet|t|testnet or custom rpc
    let cluster = "devnet";

    let solana_address = Pubkey::from_str(&response.unwrap().0).expect("Invalid public key");
    ic_cdk::println!("solana_address: {}", solana_address);

    // Get the balance
    let response: Result<(RpcResult<u64>,), _> =
        ic_cdk::call(sol_canister, "sol_getBalance", (cluster, solana_address.to_string())).await;

    let lamports = response.unwrap().0.unwrap();
    ic_cdk::println!("Balance: {} lamports", lamports);

    // Airdrop 1 SOL
    let response: Result<(RpcResult<String>,), _> = ic_cdk::call(
        sol_canister,
        "sol_requestAirdrop",
        (cluster, solana_address.to_string(), 1_000_000_000u64),
    )
    .await;
    ic_cdk::println!("Airdrop response: {:?}", response);

    let fee = 10_000;
    let amount = 1_000_000u64;

    if lamports <= amount + fee {
        ic_cdk::trap("Not enough lamports");
    }

    // Get the latest blockhash
    let response: Result<(RpcResult<String>,), _> = ic_cdk::call(sol_canister, "sol_latestBlockhash", (cluster,)).await;

    let blockhash = BlockHash::from_str(&response.unwrap().0.unwrap()).unwrap();
    ic_cdk::println!("Latest Blockhash: {:?}", blockhash);

    // Generate a transfer instruction
    let system_program_id = Pubkey::from_str("11111111111111111111111111111111").unwrap();
    let transfer_ix = Instruction::new_with_bincode(
        system_program_id,
        &(2, 0, 0, 0, 64, 66, 15, 0, 0, 0, 0, 0), // transfer 1_000_000 lamports
        vec![
            AccountMeta::new(solana_address, true),
            AccountMeta::new(
                Pubkey::from_str("AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY").unwrap(),
                false,
            ),
            AccountMeta::new(system_program_id, false),
        ],
    );

    let response: Result<(RpcResult<String>,), _> = ic_cdk::call(
        sol_canister,
        "sol_sendTransaction",
        (
            cluster,
            SendTransactionRequest {
                instructions: vec![transfer_ix.to_string()],
                recent_blockhash: Some(blockhash.to_string()),
            },
        ),
    )
    .await;

    let signature = response.unwrap().0.unwrap();
    ic_cdk::println!("Signature: {:?}", signature);

    // let transaction: Result<(RpcResult<EncodedConfirmedTransactionWithStatusMeta>,), _> =
    //     ic_cdk::call(sol_canister, "sol_getTransaction", (signature.to_string(),)).await;
    //
    // ic_cdk::println!("Transaction: {:?}", transaction);
}

/// When setting up the test canister, we need to save a reference to the solana provider canister
/// so that we can call it later.
#[ic_cdk::init]
async fn init(sol_canister: String) {
    SOL_PROVIDER_CANISTER.with(|canister| {
        *canister.borrow_mut() = Some(Principal::from_text(sol_canister).expect("Invalid principal"));
    });
}

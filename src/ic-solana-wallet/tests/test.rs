mod common;

use candid::{encode_one, Decode};
use common::{init, BASIC_IDENTITY};
use ic_agent::Agent;
use pocket_ic::PocketIcBuilder;

#[tokio::test]
async fn test_sol_address() {
    const EXPECTED_ADDRESS: &str = "5yNwm5B46JQZNeKPRnYPtjYT7yxbfgi5hCph981SnzB8";

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

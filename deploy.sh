#!/bin/bash

CANISTER_ID=solana_rpc

dfx deploy schnorr_canister

# Get the canister id
schnorr_canister_id=$(dfx canister id schnorr_canister)

# Build the solana-rpc canister
cargo build --release --target wasm32-unknown-unknown --package $CANISTER_ID
candid-extractor target/wasm32-unknown-unknown/release/${CANISTER_ID}.wasm > ${CANISTER_ID}.did

# Deploy the solana-rpc canister and set the schnorr canister id
dfx deploy $CANISTER_ID --argument "( record { nodesInSubnet = 28; schnorr_canister = opt \"${schnorr_canister_id}\" } )" --mode=reinstall -y

#dfx generate

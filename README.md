# SOLANA RPC Canister &nbsp;[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

> #### A canister for interaction between [Solana](https://solana.com) and [Internet Computer (ICP)](https://internetcomputer.org/) blockchains.

## Overview

The **SOLANA RPC canister** is an ICP canister for communicating with [Solana](https://solana.com/) using an on-chain
API.

This canister sends API requests to [JSON-RPC](https://solana.com/docs/rpc) services
using [HTTPS outcalls](https://internetcomputer.org/https-outcalls). This enables functionality similar to traditional
Solana dApps, including querying Solana smart contract states and submitting raw transactions.

The canister runs on the 28-node fiduciary subnet with the following principal: [
`bkyz2-fmaaa-aaaaa-qaaaq-cai`](https://dashboard.internetcomputer.org/canister/bkyz2-fmaaa-aaaaa-qaaaq-cai).

For information on how to verify a hash of a deployed WebAssembly module, please refer to
the [Reproducible Builds](#reproducible-builds) section.

## Quick start

Add the following to your `dfx.json` config file (replace the `ic` principal with any option from the list of the
available canisters above):

```json
{
  "canisters": {
    "ic-solana-provider": {
      "type": "custom",
      "candid": "https://github.com/mfactory-lab/ic-solana/releases/latest/download/ic-solana-provider.did",
      "wasm": "https://github.com/mfactory-lab/ic-solana/releases/latest/download/ic-solana-provider.wasm.gz",
      "remote": {
        "id": {
          "ic": "bkyz2-fmaaa-aaaaa-qaaaq-cai",
          "playground": "bkyz2-fmaaa-aaaaa-qaaaq-cai"
        }
      }
    }
  }
}
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, which then runs in the background
dfx start --clean --background

# Build and deploy your canisters to the replica and generates your candid interface
make build
```

Once the job is completed, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

### Testing

We use [PocketIC](https://github.com/dfinity/pocketic) for integration testing. Please make sure to have it installed
and the `POCKET_IC_BIN` environment variable set to the path of the `pocket-ic` binary.

You can run the tests with the following command:

```sh
make test
```

## Deployment on the Internet Computer

The canister is deployed to `bkyz2-fmaaa-aaaaa-qaaaq-cai`.

You can check the Candid UI at [
`https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=bkyz2-fmaaa-aaaaa-qaaaq-cai`](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=bkyz2-fmaaa-aaaaa-qaaaq-cai).

### Interaction with Blast Playground

You can interact with the canister using the [Blast Playground](#).

## To do

- [ ] Versioned transactions.
- [ ] Native support of threshold EdDSA.
- [ ] https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-schnorr.

## Reproducible builds

The SOLANA RPC canister
supports [reproducible builds](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/test/reproducible-builds):

1. Ensure [Docker](https://www.docker.com/get-started/) is installed on your machine.
2. Run `scripts/docker-build` in your terminal.
3. Run `sha256sum ic-solana-provider.wasm.gz` on the generated file to view the SHA-256 hash.

In order to verify the latest SOLANA RPC WASM file, please make sure to download the corresponding version of the source
code from the latest GitHub release.

## Learn More

## Credits

* [Candid interface](https://github.com/mfactory-lab/ic-solana/blob/main/src/ic-solana-provider/ic-solana-provider.did)
* This canister is monitored by [CycleOps](https://cycleops.dev).

## Related projects

* [Schnorr Signature](https://github.com/domwoe/schnorr_canister) Schnorr Signature Canister
* [Solana Galactic Bridge](https://github.com/weichain/galactic-bridge-sol): This program implements a secure deposit
  and withdrawal functionality for a Solana treasury account.
* [Bitcoin canister](https://github.com/dfinity/bitcoin-canister): interact with the Bitcoin blockchain from the
  Internet Computer.
* [ckETH](https://forum.dfinity.org/t/cketh-a-canister-issued-ether-twin-token-on-the-ic/22819): a canister-issued Ether
  twin token on the Internet Computer.
* [IC 🔗 ETH](https://github.com/dfinity/ic-eth-starter): a full-stack starter project for calling Ethereum smart
  contracts from an IC dApp.

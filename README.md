# SOLANA RPC &nbsp;[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

> #### Interact with [Solana blockchain](https://solana.com) from the [Internet Computer](https://internetcomputer.org/).

## Overview

**SOLANA RPC** is an Internet Computer canister smart contract for communicating
with [Solana](https://solana.com/) using an on-chain API.

This canister facilitates API requests to [JSON-RPC](https://solana.com/docs/rpc) services
using [HTTPS outcalls](https://internetcomputer.org/https-outcalls). This enables
functionality similar to traditional Solana dapps, including querying Solana smart contract states and submitting
raw transactions.

## Documentation

You can find extensive documentation for the SOLANA RPC canister in
the [ICP developer docs](https://internetcomputer.org/docs/current/developer-docs/multi-chain/ethereum/evm-rpc/overview).

## Canister

The SOLANA RPC canister runs on the 28-node fiduciary subnet with the following
principal: [`7hfb6-caaaa-aaaar-qadga-cai`](https://dashboard.internetcomputer.org/canister/7hfb6-caaaa-aaaar-qadga-cai).

Refer to the [Reproducible Builds](#reproducible-builds) section for information on how to verify the hash of the
deployed WebAssembly module.

## Quick Start

Add the following to your `dfx.json` config file (replace the `ic` principal with any option from the list of the
available canisters above):

```json
{
  "canisters": {
    "solana_rpc": {
      "type": "custom",
      "candid": ".../solana_rpc.did",
      "wasm": ".../solana_rpc.wasm.gz",
      "remote": {
        "id": {
          "ic": "7hfb6-caaaa-aaaar-qadga-cai"
        }
      }
    }
  }
}
```

Run the following commands to deploy the canister in your local environment:

```sh
# Start the local replica
dfx start --background

# Locally deploy the `solana_rpc` canister
dfx deploy solana_rpc
```

## Reproducible Builds

The SOLANA RPC canister
supports [reproducible builds](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/test/reproducible-builds):

1. Ensure [Docker](https://www.docker.com/get-started/) is installed on your machine.
2. Run `scripts/docker-build` in your terminal.
4. Run `sha256sum evm_rpc.wasm.gz` on the generated file to view the SHA-256 hash.

In order to verify the latest SOLANA RPC Wasm file, please make sure to download the corresponding version of the source
code from the latest GitHub release.

## Learn More

* [Candid interface](https://github.com/internet-computer-protocol/evm-rpc-canister/blob/main/candid/evm_rpc.did)

## Related Projects

* [Bitcoin canister](https://github.com/dfinity/bitcoin-canister): interact with the Bitcoin blockchain from the
  Internet Computer.
* [ckETH](https://forum.dfinity.org/t/cketh-a-canister-issued-ether-twin-token-on-the-ic/22819): a canister-issued Ether
  twin token on the Internet Computer.
* [IC 🔗 ETH](https://github.com/dfinity/ic-eth-starter): a full-stack starter project for calling Ethereum smart
  contracts from an IC dapp.
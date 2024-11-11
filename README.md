# SOLANA RPC &nbsp;[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

The **SOLANA RPC** is an Internet Computer (ICP) canister designed to facilitate communication between the [Solana](https://solana.com/) blockchain and the [Internet Computer](https://internetcomputer.org/) using an on-chain API.

This canister sends API requests to [JSON-RPC](https://solana.com/docs/rpc) services via [HTTPS outcalls](https://internetcomputer.org/https-outcalls). It enables functionalities similar to traditional Solana decentralized applications (dApps), including querying Solana smart contract states and submitting raw transactions.

The canister runs on the 34-node [fiduciary subnet](https://internetcomputer.org/docs/current/references/subnets/subnet-types#fiduciary-subnets) with the following principal: [bd3sg-teaaa-aaaaa-qaaba-cai](https://dashboard.internetcomputer.org/canister/bd3sg-teaaa-aaaaa-qaaba-cai).

Refer to the [Reproducible Builds](#reproducible-builds) section for information on how to verify a hash of a deployed WebAssembly module.

## Prerequisites

Before getting started, make sure to install the following on your machine:

- [DFINITY SDK](https://sdk.dfinity.org/docs/quickstart/local-quickstart.html)
- [Docker](https://www.docker.com/get-started/)
- [PocketIC](https://github.com/dfinity/pocketic) (for testing)

Additionally, make sure that the `POCKET_IC_BIN` environment variable is set to the path of the `pocket-ic` binary.

## Quick start

Add the following configuration to your `dfx.json` file. Replace the `ic` principal with the appropriate canister principal from the deployed canisters.

```json
{
  "canisters": {
    "solana_rpc": {
      "type": "custom",
      "candid": "https://github.com/mfactory-lab/ic-solana/releases/latest/download/ic-solana-rpc.did",
      "wasm": "https://github.com/mfactory-lab/ic-solana/releases/latest/download/ic-solana-rpc.wasm.gz",
      "remote": {
        "id": {
          "ic": "bd3sg-teaaa-aaaaa-qaaba-cai",
          "playground": "bd3sg-teaaa-aaaaa-qaaba-cai"
        }
      }
    }
  }
}
```

**Note:** Make sure to use the correct principal ID corresponding to your environment (e.g., `ic`,`playground`). Refer to the [Deployment](#deployment-on-the-internet-computer) section for more details.

## Running the project locally

To run the project locally, follow these steps:

1. Start a replica:

   ```shell
   dfx start --clean --background
   ```

   This command starts a local Internet Computer replica in the background.

2. Build and deploy canisters:

   ```shell
   dfx deploy solana_rpc --argument '(record {})'
   ```

   This command builds and deploys your canisters to the local replica and generates a Candid interface.

3. Access the application:

   Once the build and deployment are complete, your application will be accessible at:

   ```
   http://localhost:4943?canisterId={asset_canister_id}
   ```

   Replace `{asset_canister_id}` with the actual canister ID generated during deployment.

## Testing

We use [PocketIC](https://github.com/dfinity/pocketic) for integration testing. Please make sure to have it installed and the
`POCKET_IC_BIN` environment variable set to the path of the `pocket-ic` binary.

You can run the tests with the following commands:

- Run all tests:

  ```shell
  make test
  ```

- Run a specific test:

  ```shell
  make test TEST="specified_test_here"
  ```

## Deployment on the Internet Computer

The canister is deployed to `bd3sg-teaaa-aaaaa-qaaba-cai`.

You can check the Candid UI at [
`https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=bd3sg-teaaa-aaaaa-qaaba-cai`](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=bd3sg-teaaa-aaaaa-qaaba-cai).

## Reproducible builds

The SOLANA RPC canister supports [reproducible builds](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/test/reproducible-builds):

1. Ensure [Docker](https://www.docker.com/get-started/) is installed on your machine.
2. Run `./scripts/docker-build --rpc` in your terminal.
3. Run `sha256sum ic-solana-rpc.wasm.gz` on the generated file to view the SHA-256 hash.

Compare the generated SHA-256 hash with the hash provided in the repository to verify the build's integrity.

## Learn more

To learn more about the SOLANA RPC Canister and its integration with Solana and ICP, explore the following resources:

- [Candid Interface](https://github.com/mfactory-lab/ic-solana/blob/main/src/ic-solana-rpc/ic-solana-rpc.did)
- [Solana JSON-RPC API](https://solana.com/docs/rpc)
- [Internet Computer Developer Docs](https://internetcomputer.org/docs/current/developer-docs/)
- [DFINITY SDK Documentation](https://sdk.dfinity.org/docs/)
- [Internet Computer HTTPS Outcalls](https://internetcomputer.org/https-outcalls)

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).

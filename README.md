# Threshold EdDSA signature canister for Albus on IC

## Problem

Albus Protocol is currently deployed on Solana, which uses EdDSA as a digital signature scheme. The Internet Computer offers ECDSA as an alternative and doesn't support EdDSA. We needed a solution that would allow IC users to sign transactions for the Solana component of Albus on IC.

## Solution

The threshold EdDSA signature canister uses VETKD to reassemble a user's secret key and derive a key to sign transactions sent from the IC component to the Solana component of Albus on IC. This is a **temporary workaround**. Albus on IC can easily be switched to a native EdDSA threshold scheme on the Internet Computer in the future once it becomes available.

## How it works

EdDSA relies on Ed25519 curve, which takes a 32-byte seed to generate a 32-byte private key that can be used for signing. This seed can be derived through VETKD as follows:

1. Obtain an encrypted key using the `vetkd_encrypted_key` method.
2. Deserialize the encrypted key and derive a public key using the `vetkd_public_key` method.
3. Decrypt the encrypted key using the derived public key and the transport key.
4. Generate a keypair for EdDSA using the decrypted key as a seed.

Thus, by using the same 32-byte seed, the user will always get the same 32-byte private key for signing transactions with EdDSA.

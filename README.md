# Threshold EdDSA signature canister for Albus on IC

## Problem

Albus Protocol is currently deployed on Solana, which uses EdDSA as a digital signature scheme. The Internet Computer offers ECDSA as an alternative and doesn't support EdDSA. We needed a solution that would allow IC users to sign transactions for the Solana component of Albus on IC.

## Solution (temporary workaround)

The threshold EdDSA signature canister uses VETKD to reassemble a user's secret key and derive a key to sign transactions sent from the IC component to the Solana component of Albus on IC. This is a **temporary workaround**. Albus on IC can easily be switched to a native EdDSA threshold scheme on the Internet Computer in the future once it becomes available.

For details on how it works, please see comments in the code.

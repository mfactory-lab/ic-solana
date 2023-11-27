# Threshold EdDSA signature canister for Albus on IC

## Problem

Albus Protocol is currently deployed on Solana, which uses EdDSA as a digital signature scheme. The Internet Computer offers ECDSA as an alternative and doesn't support EdDSA. We needed a solution that would allow IC users to sign transactions for the Solana component of Albus on IC.

## Solution

The threshold EdDSA signature canister uses the VETKD canister to reassemble a user's secret key and derive a key to sign transactions sent from the IC component to the Solana component of Albus on IC. For details on how it works, please see comments in the code.

**Note**: this is a preliminary version developed for demonstration purposes only. It’s open to discussion and modification.

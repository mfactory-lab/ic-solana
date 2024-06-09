use crate::caller;
use crate::vetkd_types::{
    CanisterId, VetKDCurve, VetKDEncryptedKeyReply, VetKDEncryptedKeyRequest, VetKDKeyId,
    VetKDPublicKeyReply, VetKDPublicKeyRequest,
};
use ic_cdk::api;
use ic_cdk_macros::update;
use ic_crypto_internal_bls12_381_vetkd::{DerivedPublicKey, EncryptedKey};
use std::str::FromStr;

const VETKD_SYSTEM_API_CANISTER_ID: &str = "nqbbm-nqaaa-aaaak-ae5sa-cai";

fn bls12_381_test_key_1() -> VetKDKeyId {
    VetKDKeyId {
        curve: VetKDCurve::Bls12_381,
        name: "test_key_1".to_string(),
    }
}

fn vetkd_system_api_canister_id() -> CanisterId {
    use std::str::FromStr;
    CanisterId::from_str(VETKD_SYSTEM_API_CANISTER_ID).expect("failed to create canister ID")
}

// #[update]
// async fn symmetric_key_verification_key_for_note() -> String {
//     let request = VetKDPublicKeyRequest {
//         canister_id: None,
//         derivation_path: vec![b"note_symmetric_key".to_vec()],
//         key_id: bls12_381_test_key_1(),
//     };
//
//     let (response,): (VetKDPublicKeyReply,) = ic_cdk::call(
//         vetkd_system_api_canister_id(),
//         "vetkd_public_key",
//         (request,),
//     )
//         .await
//         .expect("call to vetkd_public_key failed");
//
//     hex::encode(response.public_key)
// }
//
// #[update]
// async fn encrypted_symmetric_key_for_note(
//     note_id: NoteId,
//     encryption_public_key: Vec<u8>,
// ) -> String {
//     let user_str = caller().to_string();
//     let request = NOTES.with_borrow(|notes| {
//         if let Some(note) = notes.get(&note_id) {
//             if !note.is_authorized(&user_str) {
//                 ic_cdk::trap(&format!("unauthorized key request by user {user_str}"));
//             }
//             VetKDEncryptedKeyRequest {
//                 derivation_id: {
//                     let mut buf = vec![];
//                     buf.extend_from_slice(&note_id.to_be_bytes()); // fixed-size encoding
//                     buf.extend_from_slice(note.owner.as_bytes());
//                     buf // prefix-free
//                 },
//                 public_key_derivation_path: vec![b"note_symmetric_key".to_vec()],
//                 key_id: bls12_381_test_key_1(),
//                 encryption_public_key,
//             }
//         } else {
//             ic_cdk::trap(&format!("note with ID {note_id} does not exist"));
//         }
//     });
//
//     let (response,): (VetKDEncryptedKeyReply,) = ic_cdk::call(
//         vetkd_system_api_canister_id(),
//         "vetkd_encrypted_key",
//         (request,),
//     )
//         .await
//         .expect("call to vetkd_encrypted_key failed");
//
//     hex::encode(response.encrypted_key)
// }
//

/// Retrieves a Solana keypair for the current user's principal using the VetKD (Vessel Key Derivation) system.
///
/// This function performs the following steps:
/// 1. Call the `vetkd_encrypted_key` method to get an encrypted key.
/// 2. Deserializes the encrypted key and calls the `vetkd_public_key` method to get the derived public key.
/// 3. Decrypts the encrypted key using the derived public key to get the final Solana keypair.
///
/// # Returns
///
/// An `ed25519_compact::KeyPair` representing the Solana keypair for the user's principal.
///
/// # Panics
///
/// This function panics if any of the asynchronous calls to the VetKD system or key-related operations fail.
/// Ensure that the VetKD system is reachable and responsive.

// pub async fn get_solana_keypair() -> ed25519_compact::KeyPair {
//     let user_principal = caller();
//     let derivation_path = vec![b"solana_key".to_vec()];
//
//     let request = VetKDEncryptedKeyRequest {
//         derivation_id: {
//             let mut buf = vec![];
//             buf.extend_from_slice(&note_id.to_be_bytes()); // fixed-size encoding
//             buf.extend_from_slice(note.owner.as_bytes());
//             buf // prefix-free
//         },
//         public_key_derivation_path: vec![b"note_symmetric_key".to_vec()],
//         key_id: bls12_381_test_key_1(),
//         encryption_public_key,
//     };
//
//     let (response,): (VetKDEncryptedKeyReply,) = api::call::call(
//         vetkd_system_api_canister_id(),
//         "vetkd_encrypted_key",
//         (VetKDEncryptedKeyRequest {
//             derivation_id: user_principal.as_slice().to_vec(),
//             public_key_derivation_path: derivation_path.clone(),
//             key_id: bls12_381_test_key_1(),
//             encryption_public_key: TRANSPORT_SK.public_key().serialize().into(),
//         },),
//     )
//     .await
//     .expect("call to vetkd_encrypted_key failed");
//
//     let ek = EncryptedKey::deserialize(response.encrypted_key.try_into().unwrap())
//         .expect("failed to obtain encrypted key");
//
//     let (response,): (VetKDPublicKeyReply,) = api::call::call(
//         vetkd_system_api_canister_id(),
//         "vetkd_public_key",
//         (VetKDPublicKeyRequest {
//             canister_id: None,
//             derivation_path,
//             key_id: bls12_381_test_key_1(),
//         },),
//     )
//     .await
//     .expect("call to vetkd_public_key failed");
//
//     let dpk = DerivedPublicKey::deserialize(&response.public_key).unwrap();
//     let key = TRANSPORT_SK
//         .decrypt_and_hash(&ek, &dpk, user_principal.as_ref(), 32, &[])
//         .expect("failed to decrypt symmetric key");
//
//     // ed25519_compact::KeyPair::from_seed(
//     //     ed25519_compact::Seed::from_slice(&key).expect("failed to calculate ed25519 keypair seed"),
//     // )
// }

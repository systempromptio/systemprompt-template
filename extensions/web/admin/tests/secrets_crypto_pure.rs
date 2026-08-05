#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use systemprompt_web_admin::repositories::secrets::secret_crypto::{
    SecretCryptoError, decrypt, encrypt, generate_dek, generate_nonce,
};

const KEY: [u8; 32] = [7u8; 32];
const NONCE: [u8; 12] = [3u8; 12];

#[test]
fn round_trip_recovers_the_plaintext() {
    let plaintext = b"sk-ant-api03-not-a-real-key";
    let sealed = encrypt(&KEY, &NONCE, plaintext).expect("encrypt");
    assert_ne!(sealed.as_slice(), plaintext.as_slice());
    let opened = decrypt(&KEY, &NONCE, &sealed).expect("decrypt");
    assert_eq!(opened, plaintext.to_vec());
}

#[test]
fn empty_plaintext_round_trips() {
    let sealed = encrypt(&KEY, &NONCE, b"").expect("encrypt");
    assert_eq!(sealed.len(), 16, "an empty body is just the auth tag");
    assert_eq!(
        decrypt(&KEY, &NONCE, &sealed).expect("decrypt"),
        Vec::<u8>::new()
    );
}

#[test]
fn ciphertext_is_the_plaintext_length_plus_a_tag() {
    let sealed = encrypt(&KEY, &NONCE, b"0123456789").expect("encrypt");
    assert_eq!(sealed.len(), 10 + 16);
}

#[test]
fn the_same_key_and_nonce_are_deterministic() {
    let a = encrypt(&KEY, &NONCE, b"same input").expect("encrypt");
    let b = encrypt(&KEY, &NONCE, b"same input").expect("encrypt");
    assert_eq!(a, b);
}

#[test]
fn a_different_nonce_produces_different_ciphertext() {
    let other = [9u8; 12];
    let a = encrypt(&KEY, &NONCE, b"same input").expect("encrypt");
    let b = encrypt(&KEY, &other, b"same input").expect("encrypt");
    assert_ne!(a, b);
}

#[test]
fn decrypting_with_the_wrong_key_fails() {
    let sealed = encrypt(&KEY, &NONCE, b"secret").expect("encrypt");
    let wrong = [8u8; 32];
    assert!(matches!(
        decrypt(&wrong, &NONCE, &sealed),
        Err(SecretCryptoError::DecryptionFailed(_))
    ));
}

#[test]
fn decrypting_with_the_wrong_nonce_fails() {
    let sealed = encrypt(&KEY, &NONCE, b"secret").expect("encrypt");
    assert!(matches!(
        decrypt(&KEY, &[4u8; 12], &sealed),
        Err(SecretCryptoError::DecryptionFailed(_))
    ));
}

#[test]
fn a_tampered_ciphertext_is_rejected() {
    let mut sealed = encrypt(&KEY, &NONCE, b"secret").expect("encrypt");
    sealed[0] ^= 0xff;
    assert!(matches!(
        decrypt(&KEY, &NONCE, &sealed),
        Err(SecretCryptoError::DecryptionFailed(_))
    ));
}

#[test]
fn a_truncated_ciphertext_is_rejected() {
    assert!(decrypt(&KEY, &NONCE, b"too short").is_err());
    assert!(decrypt(&KEY, &NONCE, b"").is_err());
}

#[test]
fn generated_keys_and_nonces_are_not_constant() {
    assert_ne!(generate_dek(), generate_dek());
    assert_ne!(generate_dek(), [0u8; 32]);
    assert_ne!(generate_nonce(), generate_nonce());
    assert_ne!(generate_nonce(), [0u8; 12]);
}

#[test]
fn a_generated_dek_seals_and_opens_its_own_traffic() {
    let dek = generate_dek();
    let nonce = generate_nonce();
    let sealed = encrypt(&dek, &nonce, b"per-user secret").expect("encrypt");
    assert_eq!(
        decrypt(&dek, &nonce, &sealed).expect("decrypt"),
        b"per-user secret".to_vec()
    );
}

#[test]
fn error_messages_name_the_failing_stage() {
    assert_eq!(
        SecretCryptoError::MasterKeyMissing.to_string(),
        "Master key not configured"
    );
    assert_eq!(
        SecretCryptoError::InvalidKeyMaterial.to_string(),
        "Invalid key material"
    );
    assert_eq!(
        SecretCryptoError::Database("pool closed".into()).to_string(),
        "Database error: pool closed"
    );
    assert_eq!(
        SecretCryptoError::EncryptionFailed("aead".into()).to_string(),
        "Encryption failed: aead"
    );
    assert_eq!(
        SecretCryptoError::DecryptionFailed("aead".into()).to_string(),
        "Decryption failed: aead"
    );
}

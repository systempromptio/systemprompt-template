//! ChaCha20-Poly1305 primitives for secret storage, and the key wrap.
//!
//! A user's data encryption key is wrapped under the web member's conferred
//! cipher (`MemberCipher`): a key derived for this member, which no other
//! member's cipher opens. The member never holds the master key.

use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use systemprompt::traits::{MemberCipher, MemberCipherError};

#[derive(Debug, thiserror::Error)]
pub enum SecretCryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key wrap failed")]
    Wrap(#[source] MemberCipherError),
    #[error("Invalid key material")]
    InvalidKeyMaterial,
    #[error("Database error: {0}")]
    Database(String),
}

#[must_use]
pub fn generate_dek() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[must_use]
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn encrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
) -> Result<Vec<u8>, SecretCryptoError> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from(*nonce);
    cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| SecretCryptoError::EncryptionFailed(e.to_string()))
}

pub fn decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
) -> Result<Vec<u8>, SecretCryptoError> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from(*nonce);
    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| SecretCryptoError::DecryptionFailed(e.to_string()))
}

pub fn wrap_dek(cipher: &dyn MemberCipher, dek: &[u8; 32]) -> Result<Vec<u8>, SecretCryptoError> {
    seal_bytes(cipher, dek)
}

pub fn unwrap_dek(
    cipher: &dyn MemberCipher,
    wrapped: &[u8],
) -> Result<[u8; 32], SecretCryptoError> {
    open_bytes(cipher, wrapped)?
        .try_into()
        .map_err(|v: Vec<u8>| {
            tracing::warn!(len = v.len(), "Unwrapped DEK is not 32 bytes");
            SecretCryptoError::InvalidKeyMaterial
        })
}

pub fn seal_bytes(
    cipher: &dyn MemberCipher,
    plaintext: &[u8],
) -> Result<Vec<u8>, SecretCryptoError> {
    cipher
        .seal(&hex::encode(plaintext))
        .map(String::into_bytes)
        .map_err(SecretCryptoError::Wrap)
}

pub fn open_bytes(cipher: &dyn MemberCipher, sealed: &[u8]) -> Result<Vec<u8>, SecretCryptoError> {
    let sealed = std::str::from_utf8(sealed).map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;
    let hex_plain = cipher.open(sealed).map_err(SecretCryptoError::Wrap)?;
    hex::decode(hex_plain).map_err(|_| SecretCryptoError::InvalidKeyMaterial)
}

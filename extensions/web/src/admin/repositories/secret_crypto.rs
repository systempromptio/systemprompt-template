use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};

#[derive(Debug, thiserror::Error)]
pub enum SecretCryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Master key not configured")]
    MasterKeyMissing,
    #[error("Invalid key material")]
    InvalidKeyMaterial,
    #[error("User DEK not found for user {0}")]
    DekNotFound(String),
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
    let nonce = Nonce::from_slice(nonce);
    cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| SecretCryptoError::EncryptionFailed(e.to_string()))
}

pub fn decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
) -> Result<Vec<u8>, SecretCryptoError> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| SecretCryptoError::DecryptionFailed(e.to_string()))
}

pub fn load_master_key() -> Result<[u8; 32], SecretCryptoError> {
    let hex_key = std::env::var("ENCRYPTION_MASTER_KEY")
        .ok()
        .or_else(|| {
            systemprompt::models::SecretsBootstrap::get()
                .ok()
                .and_then(|s| s.get("encryption_master_key").cloned())
        })
        .ok_or(SecretCryptoError::MasterKeyMissing)?;

    let bytes = hex::decode(hex_key.trim()).map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;

    let key: [u8; 32] = bytes
        .try_into()
        .map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;

    Ok(key)
}

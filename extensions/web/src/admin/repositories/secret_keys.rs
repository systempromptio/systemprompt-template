use std::sync::Arc;

use sqlx::PgPool;

use super::secret_crypto::{self, SecretCryptoError};

pub async fn get_or_create_user_dek(
    pool: &Arc<PgPool>,
    user_id: &str,
    master_key: &[u8; 32],
) -> Result<[u8; 32], SecretCryptoError> {
    let row: Option<(Vec<u8>, Vec<u8>)> = sqlx::query_as(
        "SELECT encrypted_dek, dek_nonce FROM user_encryption_keys WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    if let Some((encrypted_dek, dek_nonce)) = row {
        let nonce: [u8; 12] = dek_nonce
            .try_into()
            .map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;
        let plaintext = secret_crypto::decrypt(master_key, &nonce, &encrypted_dek)?;
        let dek: [u8; 32] = plaintext
            .try_into()
            .map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;
        return Ok(dek);
    }

    let dek = secret_crypto::generate_dek();
    let nonce = secret_crypto::generate_nonce();
    let encrypted_dek = secret_crypto::encrypt(master_key, &nonce, &dek)?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO user_encryption_keys (id, user_id, encrypted_dek, dek_nonce) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&encrypted_dek)
    .bind(nonce.as_slice())
    .execute(pool.as_ref())
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    tracing::info!(user_id = %user_id, "Created new encryption key for user");

    Ok(dek)
}

pub async fn rotate_user_dek(
    pool: &Arc<PgPool>,
    user_id: &str,
    master_key: &[u8; 32],
) -> Result<(), SecretCryptoError> {
    let old_dek = get_or_create_user_dek(pool, user_id, master_key).await?;

    let new_dek = secret_crypto::generate_dek();
    let new_dek_nonce = secret_crypto::generate_nonce();
    let encrypted_new_dek = secret_crypto::encrypt(master_key, &new_dek_nonce, &new_dek)?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    let secret_rows: Vec<(String, Vec<u8>, Vec<u8>)> = sqlx::query_as(
        "SELECT id, encrypted_value, value_nonce FROM plugin_env_vars \
         WHERE user_id = $1 AND is_secret = true AND encrypted_value IS NOT NULL",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    for (row_id, encrypted_value, value_nonce) in &secret_rows {
        let old_nonce: [u8; 12] = value_nonce
            .as_slice()
            .try_into()
            .map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;
        let plaintext = secret_crypto::decrypt(&old_dek, &old_nonce, encrypted_value)?;

        let new_value_nonce = secret_crypto::generate_nonce();
        let new_encrypted = secret_crypto::encrypt(&new_dek, &new_value_nonce, &plaintext)?;

        sqlx::query(
            "UPDATE plugin_env_vars SET encrypted_value = $1, value_nonce = $2, \
             key_version = key_version + 1 WHERE id = $3",
        )
        .bind(&new_encrypted)
        .bind(new_value_nonce.as_slice())
        .bind(row_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| SecretCryptoError::Database(e.to_string()))?;
    }

    sqlx::query(
        "UPDATE user_encryption_keys SET encrypted_dek = $1, dek_nonce = $2, \
         key_version = key_version + 1, rotated_at = NOW() WHERE user_id = $3",
    )
    .bind(&encrypted_new_dek)
    .bind(new_dek_nonce.as_slice())
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    tracing::info!(
        user_id = %user_id,
        secrets_rotated = %secret_rows.len(),
        "Rotated user encryption key"
    );

    Ok(())
}

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::secret_crypto::{self, SecretCryptoError};

pub async fn get_or_create_user_dek(
    pool: &PgPool,
    user_id: &UserId,
    master_key: &[u8; 32],
) -> Result<[u8; 32], SecretCryptoError> {
    let row = sqlx::query!(
        "SELECT encrypted_dek, dek_nonce FROM user_encryption_keys WHERE user_id = $1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    if let Some(row) = row {
        let nonce: [u8; 12] = row
            .dek_nonce
            .try_into()
            .map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;
        let plaintext = secret_crypto::decrypt(master_key, &nonce, &row.encrypted_dek)?;
        let dek: [u8; 32] = plaintext
            .try_into()
            .map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;
        return Ok(dek);
    }

    let dek = secret_crypto::generate_dek();
    let nonce = secret_crypto::generate_nonce();
    let encrypted_dek = secret_crypto::encrypt(master_key, &nonce, &dek)?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO user_encryption_keys (id, user_id, encrypted_dek, dek_nonce) \
         VALUES ($1, $2, $3, $4)",
        id,
        user_id.as_str(),
        encrypted_dek.as_slice(),
        nonce.as_slice(),
    )
    .execute(pool)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    tracing::info!(user_id = %user_id, "Created new encryption key for user");

    Ok(dek)
}

pub async fn rotate_user_dek(
    pool: &PgPool,
    user_id: &UserId,
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

    let secret_rows = sqlx::query!(
        "SELECT id, encrypted_value, value_nonce FROM plugin_env_vars \
         WHERE user_id = $1 AND is_secret = true AND encrypted_value IS NOT NULL",
        user_id.as_str(),
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    for row in &secret_rows {
        let old_nonce: [u8; 12] = row
            .value_nonce
            .as_deref()
            .unwrap_or(&[])
            .try_into()
            .map_err(|_| SecretCryptoError::InvalidKeyMaterial)?;
        let encrypted_value = row.encrypted_value.as_deref().unwrap_or(&[]);
        let plaintext = secret_crypto::decrypt(&old_dek, &old_nonce, encrypted_value)?;

        let new_value_nonce = secret_crypto::generate_nonce();
        let new_encrypted = secret_crypto::encrypt(&new_dek, &new_value_nonce, &plaintext)?;

        sqlx::query!(
            "UPDATE plugin_env_vars SET encrypted_value = $1, value_nonce = $2, \
             key_version = key_version + 1 WHERE id = $3",
            new_encrypted.as_slice(),
            new_value_nonce.as_slice(),
            row.id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| SecretCryptoError::Database(e.to_string()))?;
    }

    sqlx::query!(
        "UPDATE user_encryption_keys SET encrypted_dek = $1, dek_nonce = $2, \
         key_version = key_version + 1, rotated_at = NOW() WHERE user_id = $3",
        encrypted_new_dek.as_slice(),
        new_dek_nonce.as_slice(),
        user_id.as_str(),
    )
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

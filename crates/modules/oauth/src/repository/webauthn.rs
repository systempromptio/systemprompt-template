use anyhow::Result;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct WebAuthnCredential {
    pub id: String,
    pub user_id: String,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: u32,
    pub display_name: String,
    pub device_type: String,
    pub transports: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl crate::repository::OAuthRepository {
    pub async fn store_webauthn_credential(
        &self,
        id: &str,
        user_id: &str,
        credential_id: &[u8],
        public_key: &[u8],
        counter: u32,
        display_name: &str,
        device_type: &str,
        transports: &[String],
    ) -> Result<()> {
        let transports_json = serde_json::to_string(transports)?;
        let counter_i32 = counter as i32;
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO webauthn_credentials
             (id, user_id, credential_id, public_key, counter, display_name, device_type,
             transports, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            id,
            user_id,
            credential_id,
            public_key,
            counter_i32,
            display_name,
            device_type,
            transports_json,
            now
        )
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    pub async fn get_webauthn_credentials(&self, user_id: &str) -> Result<Vec<WebAuthnCredential>> {
        let rows = sqlx::query!(
            "SELECT id, user_id, credential_id, public_key, counter, display_name,
                    device_type, transports, created_at, last_used_at
             FROM webauthn_credentials WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(self.pool_ref())
        .await?;

        let credentials = rows
            .into_iter()
            .map(|row| {
                let transports: Vec<String> = row
                    .transports
                    .as_ref()
                    .and_then(|t| serde_json::from_str(t).ok())
                    .unwrap_or_else(|| vec!["internal".to_string()]);
                WebAuthnCredential {
                    id: row.id,
                    user_id: row.user_id,
                    credential_id: row.credential_id,
                    public_key: row.public_key,
                    counter: row.counter as u32,
                    display_name: row.display_name,
                    device_type: row.device_type.unwrap_or_else(|| "platform".to_string()),
                    transports,
                    created_at: row.created_at,
                    last_used_at: row.last_used_at,
                }
            })
            .collect();

        Ok(credentials)
    }

    pub async fn update_webauthn_credential_counter(
        &self,
        credential_id: &[u8],
        counter: u32,
    ) -> Result<()> {
        let counter_i32 = counter as i32;
        let now = Utc::now();

        sqlx::query!(
            "UPDATE webauthn_credentials SET counter = $1, last_used_at = $2
             WHERE credential_id = $3",
            counter_i32,
            now,
            credential_id
        )
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }
}

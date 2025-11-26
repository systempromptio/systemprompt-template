use anyhow::Result;
use chrono::NaiveDateTime;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, JsonRow};
use systemprompt_traits::Repository as _;

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
    pub created_at: NaiveDateTime,
    pub last_used_at: Option<NaiveDateTime>,
}

impl WebAuthnCredential {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing user_id"))?
            .to_string();

        let credential_id = row
            .get("credential_id")
            .and_then(|v| {
                use base64::{engine::general_purpose::STANDARD, Engine};
                if let Some(s) = v.as_str() {
                    STANDARD.decode(s).ok()
                } else {
                    serde_json::from_value(v.clone()).ok()
                }
            })
            .ok_or_else(|| anyhow!("Missing or invalid credential_id"))?;

        let public_key = row
            .get("public_key")
            .and_then(|v| {
                use base64::{engine::general_purpose::STANDARD, Engine};
                if let Some(s) = v.as_str() {
                    STANDARD.decode(s).ok()
                } else {
                    serde_json::from_value(v.clone()).ok()
                }
            })
            .ok_or_else(|| anyhow!("Missing or invalid public_key"))?;

        let counter = row
            .get("counter")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing counter"))? as u32;

        let display_name = row
            .get("display_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing display_name"))?
            .to_string();

        let device_type = row
            .get("device_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing device_type"))?
            .to_string();

        let transports = row
            .get("transports")
            .and_then(|v| {
                if let Some(arr) = v.as_array() {
                    Some(
                        arr.iter()
                            .filter_map(|t| t.as_str().map(ToString::to_string))
                            .collect(),
                    )
                } else if let Some(s) = v.as_str() {
                    serde_json::from_str::<Vec<String>>(s).ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| vec!["internal".to_string()]);

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid created_at field"))?
            .naive_utc();

        let last_used_at = row
            .get("last_used_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .map(|dt| dt.naive_utc());

        Ok(Self {
            id,
            user_id,
            credential_id,
            public_key,
            counter,
            display_name,
            device_type,
            transports,
            created_at,
            last_used_at,
        })
    }
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
        let counter_i64 = i64::from(counter);
        let transports_json = serde_json::to_string(transports)?;

        self.pool()
            .as_ref()
            .execute(
                &DatabaseQueryEnum::InsertCredential.get(self.pool().as_ref()),
                &[
                    &id,
                    &user_id,
                    &credential_id,
                    &public_key,
                    &counter_i64,
                    &display_name,
                    &device_type,
                    &transports_json,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get_webauthn_credentials(&self, user_id: &str) -> Result<Vec<WebAuthnCredential>> {
        let rows = self
            .pool()
            .as_ref()
            .fetch_all(
                &DatabaseQueryEnum::GetCredentialsByUserId.get(self.pool().as_ref()),
                &[&user_id],
            )
            .await?;

        let credentials = rows
            .into_iter()
            .map(|row| WebAuthnCredential::from_json_row(&row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(credentials)
    }

    pub async fn update_webauthn_credential_counter(
        &self,
        credential_id: &[u8],
        counter: u32,
    ) -> Result<()> {
        let now = chrono::Utc::now();
        let counter_i64 = i64::from(counter);

        self.db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::UpdateCredentialCounter.get(self.db_pool.as_ref()),
                &[&counter_i64, &now, &credential_id],
            )
            .await?;

        Ok(())
    }
}

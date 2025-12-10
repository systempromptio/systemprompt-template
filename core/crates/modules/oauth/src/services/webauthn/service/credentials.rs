use super::WebAuthnService;
use anyhow::Result;
use uuid::Uuid;
use webauthn_rs::prelude::*;

impl WebAuthnService {
    pub(super) async fn store_credential(
        &self,
        user_id: &str,
        sk: &Passkey,
        display_name: &str,
    ) -> Result<()> {
        let credential_id = sk.cred_id().clone();
        let public_key = serde_json::to_vec(sk)?;
        let counter = 0u32;

        let transports: Vec<String> = {
            let passkey_json = serde_json::to_value(sk)?;
            passkey_json
                .get("cred")
                .and_then(|cred| cred.get("transports"))
                .and_then(|t| t.as_array())
                .map_or_else(
                    || vec!["internal".to_string()],
                    |arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(str::to_lowercase))
                            .collect()
                    },
                )
        };

        self.oauth_repo
            .store_webauthn_credential(
                &Uuid::new_v4().to_string(),
                user_id,
                &credential_id,
                &public_key,
                counter,
                display_name,
                "platform",
                &transports,
            )
            .await
    }

    pub(super) async fn get_user_credentials(&self, user_id: &str) -> Result<Vec<Passkey>> {
        let credentials = self.oauth_repo.get_webauthn_credentials(user_id).await?;

        let mut passkeys = Vec::new();
        for cred in credentials {
            let mut passkey_json: serde_json::Value = serde_json::from_slice(&cred.public_key)?;

            if let Some(credential) = passkey_json.get_mut("cred") {
                let transports_json: Vec<String> = cred
                    .transports
                    .iter()
                    .map(|t| {
                        t.to_lowercase()
                            .replace("internal", "Internal")
                            .replace("usb", "Usb")
                            .replace("nfc", "Nfc")
                            .replace("ble", "Ble")
                            .replace("hybrid", "Hybrid")
                    })
                    .collect();

                credential["transports"] = serde_json::json!(transports_json);
            }

            let passkey: Passkey = serde_json::from_value(passkey_json)?;
            passkeys.push(passkey);
        }

        Ok(passkeys)
    }

    pub(super) async fn get_user_credentials_by_email(&self, email: &str) -> Result<Vec<Passkey>> {
        if let Some(user) = self
            .user_creation_service
            .user_repo
            .find_by_email(email)
            .await?
        {
            self.get_user_credentials(user.id.as_ref()).await
        } else {
            Ok(Vec::new())
        }
    }

    pub(super) async fn update_credential_counter(
        &self,
        credential_id: &[u8],
        counter: u32,
    ) -> Result<()> {
        self.oauth_repo
            .update_webauthn_credential_counter(credential_id, counter)
            .await
    }
}

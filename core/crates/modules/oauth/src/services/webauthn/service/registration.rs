use super::WebAuthnService;
use anyhow::Result;
use base64::engine::{general_purpose, Engine};
use std::time::Instant;
use systemprompt_core_logging::LogLevel;
use uuid::Uuid;
use webauthn_rs::prelude::*;

impl WebAuthnService {
    pub async fn start_registration(
        &self,
        username: &str,
        email: &str,
        full_name: Option<&str>,
    ) -> Result<(CreationChallengeResponse, String)> {
        let user_unique_id = Uuid::new_v4();
        let display_name = full_name.filter(|n| !n.is_empty()).unwrap_or(username);

        let exclude_credentials = self.get_user_credentials_by_email(email).await?;

        let exclude_cred_ids: Vec<_> = exclude_credentials
            .iter()
            .map(|pk| pk.cred_id().clone())
            .collect();

        let exclude_cred_ids_len = exclude_cred_ids.len();

        let (ccr, reg_state) = self.webauthn.start_passkey_registration(
            user_unique_id,
            username,
            display_name,
            if exclude_cred_ids.is_empty() {
                None
            } else {
                Some(exclude_cred_ids)
            },
        )?;

        let challenge_id = Uuid::new_v4().to_string();

        {
            let mut states = self.reg_states.lock().await;
            states.insert(challenge_id.clone(), (reg_state, Instant::now()));
        }

        self.log_service
            .log(
                LogLevel::Info,
                "webauthn",
                "Registration ceremony initiated",
                Some(serde_json::json!({
                    "username": username,
                    "user_email": email,
                    "challenge_id": challenge_id,
                    "user_unique_id": user_unique_id,
                    "display_name": display_name,
                    "full_name": full_name,
                    "excluded_credentials_count": exclude_cred_ids_len
                })),
            )
            .await?;

        Ok((ccr, challenge_id))
    }

    pub async fn finish_registration(
        &self,
        challenge_id: &str,
        username: &str,
        email: &str,
        full_name: Option<&str>,
        reg_response: &RegisterPublicKeyCredential,
    ) -> Result<String> {
        let reg_state = self
            .retrieve_and_remove_registration_state(challenge_id)
            .await?;

        match self
            .webauthn
            .finish_passkey_registration(reg_response, &reg_state)
        {
            Ok(sk) => {
                let user_id = self
                    .user_creation_service
                    .create_user_with_webauthn_registration(username, email, full_name)
                    .await?;

                let credential_id = sk.cred_id().clone();
                let display_name = full_name.filter(|n| !n.is_empty()).unwrap_or(username);
                self.complete_registration(&user_id, &sk, display_name, challenge_id, email)
                    .await?;

                self.log_service
                    .log(
                        LogLevel::Info,
                        "webauthn",
                        "WebAuthn registration completed",
                        Some(serde_json::json!({
                            "username": username,
                            "user_email": email,
                            "user_id": user_id,
                            "challenge_id": challenge_id,
                            "credential_id": general_purpose::STANDARD.encode(&credential_id),
                            "display_name": display_name,
                            "full_name": full_name,
                            "counter": 0
                        })),
                    )
                    .await?;

                Ok(user_id)
            },
            Err(e) => {
                self.log_service
                    .log(
                        LogLevel::Info,
                        "webauthn",
                        "WebAuthn registration failed",
                        Some(serde_json::json!({
                            "username": username,
                            "user_email": email,
                            "challenge_id": challenge_id,
                            "failure_reason": e.to_string(),
                            "full_name": full_name
                        })),
                    )
                    .await
                    .ok();
                Err(e.into())
            },
        }
    }

    async fn retrieve_and_remove_registration_state(
        &self,
        challenge_id: &str,
    ) -> Result<PasskeyRegistration> {
        let mut states = self.reg_states.lock().await;
        states
            .remove(challenge_id)
            .map(|(state, _timestamp)| state)
            .ok_or_else(|| anyhow::anyhow!("Registration state not found or expired"))
    }

    async fn complete_registration(
        &self,
        user_id: &str,
        sk: &Passkey,
        display_name: &str,
        _challenge_id: &str,
        _email: &str,
    ) -> Result<()> {
        self.store_credential(user_id, sk, display_name).await?;
        Ok(())
    }
}

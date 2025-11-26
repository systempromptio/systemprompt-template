use super::config::WebAuthnConfig;
use super::user_service::UserCreationService;
use crate::repository::OAuthRepository;
use anyhow::Result;
use base64::engine::{general_purpose, Engine};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_users::repository::UserRepository;
use tokio::sync::Mutex;
use uuid::Uuid;
use webauthn_rs::prelude::*;
use webauthn_rs::{Webauthn, WebauthnBuilder};

#[derive(Debug)]
struct AuthenticationStateData {
    state: PasskeyAuthentication,
    user_id: String,
    oauth_state: Option<String>,
    timestamp: Instant,
}

#[derive(Debug)]
pub struct WebAuthnService {
    webauthn: Webauthn,
    config: WebAuthnConfig,
    oauth_repo: OAuthRepository,
    user_creation_service: UserCreationService,
    log_service: LogService,
    reg_states: Arc<Mutex<HashMap<String, (PasskeyRegistration, Instant)>>>,
    auth_states: Arc<Mutex<HashMap<String, AuthenticationStateData>>>,
}

impl WebAuthnService {
    pub fn new(
        oauth_repo: OAuthRepository,
        user_repo: UserRepository,
        log_service: LogService,
    ) -> Result<Self> {
        Self::with_config(
            WebAuthnConfig::default(),
            oauth_repo,
            user_repo,
            log_service,
        )
    }

    pub fn with_config(
        config: WebAuthnConfig,
        oauth_repo: OAuthRepository,
        user_repo: UserRepository,
        log_service: LogService,
    ) -> Result<Self> {
        let webauthn = WebauthnBuilder::new(&config.rp_id, &config.rp_origin)?
            .rp_name(&config.rp_name)
            .allow_any_port(config.allow_any_port)
            .allow_subdomains(config.allow_subdomains)
            .build()?;

        let user_creation_service = UserCreationService::new(user_repo);

        Ok(Self {
            webauthn,
            config,
            oauth_repo,
            user_creation_service,
            log_service,
            reg_states: Arc::new(Mutex::new(HashMap::new())),
            auth_states: Arc::new(Mutex::new(HashMap::new())),
        })
    }

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

    pub async fn start_authentication(
        &self,
        email: &str,
        oauth_state: Option<String>,
    ) -> Result<(RequestChallengeResponse, String)> {
        let user = self
            .user_creation_service
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let user_credentials = self.get_user_credentials(&user.uuid).await?;

        if user_credentials.is_empty() {
            return Err(anyhow::anyhow!("No credentials found for user"));
        }

        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(&user_credentials)?;

        let challenge_id = Uuid::new_v4().to_string();

        // Store authentication state in memory with metadata
        {
            let mut states = self.auth_states.lock().await;
            states.insert(
                challenge_id.clone(),
                AuthenticationStateData {
                    state: auth_state,
                    user_id: user.uuid.clone(),
                    oauth_state: oauth_state.clone(),
                    timestamp: Instant::now(),
                },
            );
        }

        // Challenge metadata now stored in memory with authentication state

        self.log_service
            .log(
                LogLevel::Info,
                "webauthn",
                "Authentication ceremony initiated",
                Some(serde_json::json!({
                    "user_email": email,
                    "user_id": user.uuid,
                    "challenge_id": challenge_id,
                    "available_credentials": user_credentials.len(),
                    "oauth_state_present": oauth_state.is_some()
                })),
            )
            .await?;

        Ok((rcr, challenge_id))
    }

    pub async fn finish_authentication(
        &self,
        challenge_id: &str,
        auth_response: &PublicKeyCredential,
    ) -> Result<(String, Option<String>)> {
        let (auth_state, user_id, oauth_state) = self
            .retrieve_and_remove_authentication_state(challenge_id)
            .await?;

        match self
            .webauthn
            .finish_passkey_authentication(auth_response, &auth_state)
        {
            Ok(auth_result) => {
                self.complete_authentication(&auth_result, challenge_id)
                    .await?;

                self.log_service.log(LogLevel::Info, "webauthn", "WebAuthn authentication successful", Some(serde_json::json!({
                        "user_id": user_id,
                        "challenge_id": challenge_id,
                        "credential_id": general_purpose::STANDARD.encode(auth_result.cred_id().as_ref()),
                        "counter": auth_result.counter(),
                        "oauth_state_present": oauth_state.is_some()
                    }))).await?;

                Ok((user_id, oauth_state))
            },
            Err(e) => {
                self.log_service
                    .log(
                        LogLevel::Info,
                        "webauthn",
                        "WebAuthn authentication failed",
                        Some(serde_json::json!({
                            "user_id": user_id,
                            "challenge_id": challenge_id,
                            "failure_reason": e.to_string(),
                            "attempt_count": 1
                        })),
                    )
                    .await
                    .ok();

                Err(e.into())
            },
        }
    }

    async fn retrieve_and_remove_authentication_state(
        &self,
        challenge_id: &str,
    ) -> Result<(PasskeyAuthentication, String, Option<String>)> {
        let mut states = self.auth_states.lock().await;
        states
            .remove(challenge_id)
            .map(|data| (data.state, data.user_id, data.oauth_state))
            .ok_or_else(|| anyhow::anyhow!("Authentication state not found or expired"))
    }

    async fn complete_authentication(
        &self,
        auth_result: &AuthenticationResult,
        _challenge_id: &str,
    ) -> Result<()> {
        let cred_id = auth_result.cred_id();
        self.update_credential_counter(cred_id.as_ref(), auth_result.counter())
            .await?;
        Ok(())
    }

    async fn store_credential(
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

    async fn get_user_credentials(&self, user_id: &str) -> Result<Vec<Passkey>> {
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

    async fn get_user_credentials_by_email(&self, email: &str) -> Result<Vec<Passkey>> {
        if let Some(user) = self
            .user_creation_service
            .user_repo
            .find_by_email(email)
            .await?
        {
            self.get_user_credentials(&user.uuid).await
        } else {
            Ok(Vec::new())
        }
    }

    async fn update_credential_counter(&self, credential_id: &[u8], counter: u32) -> Result<()> {
        self.oauth_repo
            .update_webauthn_credential_counter(credential_id, counter)
            .await
    }

    pub async fn cleanup_expired_states(&self) -> Result<()> {
        let now = Instant::now();
        let expiry_duration = self.config.challenge_expiry;

        // Clean up expired registration states
        {
            let mut reg_states = self.reg_states.lock().await;
            reg_states.retain(|_challenge_id, (_state, timestamp)| {
                now.duration_since(*timestamp) < expiry_duration
            });
        }

        // Clean up expired authentication states
        {
            let mut auth_states = self.auth_states.lock().await;
            auth_states
                .retain(|_challenge_id, data| now.duration_since(data.timestamp) < expiry_duration);
        }

        Ok(())
    }
}

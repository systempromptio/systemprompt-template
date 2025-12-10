mod authentication;
mod credentials;
mod registration;

use super::config::WebAuthnConfig;
use super::user_service::UserCreationService;
use crate::repository::OAuthRepository;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use systemprompt_core_logging::LogService;
use systemprompt_core_users::repository::UserRepository;
use tokio::sync::Mutex;
use webauthn_rs::prelude::*;
use webauthn_rs::{Webauthn, WebauthnBuilder};

#[derive(Debug)]
pub(super) struct AuthenticationStateData {
    pub state: PasskeyAuthentication,
    pub user_id: String,
    pub oauth_state: Option<String>,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct WebAuthnService {
    pub(super) webauthn: Webauthn,
    pub(super) config: WebAuthnConfig,
    pub(super) oauth_repo: OAuthRepository,
    pub(super) user_creation_service: UserCreationService,
    pub(super) log_service: LogService,
    pub(super) reg_states: Arc<Mutex<HashMap<String, (PasskeyRegistration, Instant)>>>,
    pub(super) auth_states: Arc<Mutex<HashMap<String, AuthenticationStateData>>>,
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

    pub async fn cleanup_expired_states(&self) -> Result<()> {
        let now = Instant::now();
        let expiry_duration = self.config.challenge_expiry;

        {
            let mut reg_states = self.reg_states.lock().await;
            reg_states.retain(|_challenge_id, (_state, timestamp)| {
                now.duration_since(*timestamp) < expiry_duration
            });
        }

        {
            let mut auth_states = self.auth_states.lock().await;
            auth_states
                .retain(|_challenge_id, data| now.duration_since(data.timestamp) < expiry_duration);
        }

        Ok(())
    }
}

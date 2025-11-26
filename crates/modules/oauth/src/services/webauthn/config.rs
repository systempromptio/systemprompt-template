use std::time::Duration;
use webauthn_rs::prelude::*;

#[derive(Debug, Clone)]
pub struct WebAuthnConfig {
    pub rp_id: String,
    pub rp_origin: Url,
    pub rp_name: String,
    pub challenge_expiry: Duration,
    pub allow_any_port: bool,
    pub allow_subdomains: bool,
}

impl Default for WebAuthnConfig {
    fn default() -> Self {
        let config = systemprompt_core_system::Config::global();
        let api_url = &config.api_external_url;
        let parsed_url = Url::parse(api_url).expect("API_EXTERNAL_URL must be valid");

        let rp_id = parsed_url
            .host_str()
            .expect("API_EXTERNAL_URL must contain a valid host for WebAuthn RP ID")
            .to_string();

        Self {
            rp_id,
            rp_origin: parsed_url,
            rp_name: format!("{} OAuth", config.sitename),
            challenge_expiry: Duration::from_secs(300), // 5 minutes
            allow_any_port: true,
            allow_subdomains: true,
        }
    }
}

impl WebAuthnConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rp_id(mut self, rp_id: impl Into<String>) -> Self {
        self.rp_id = rp_id.into();
        self
    }

    pub fn with_rp_origin(mut self, rp_origin: Url) -> Self {
        self.rp_origin = rp_origin;
        self
    }

    pub fn with_rp_name(mut self, rp_name: impl Into<String>) -> Self {
        self.rp_name = rp_name.into();
        self
    }

    pub const fn with_challenge_expiry(mut self, expiry: Duration) -> Self {
        self.challenge_expiry = expiry;
        self
    }

    pub const fn with_any_port(mut self, allow: bool) -> Self {
        self.allow_any_port = allow;
        self
    }

    pub const fn with_subdomains(mut self, allow: bool) -> Self {
        self.allow_subdomains = allow;
        self
    }

    pub fn challenge_expiry_chrono(&self) -> chrono::Duration {
        chrono::Duration::from_std(self.challenge_expiry)
            .unwrap_or_else(|_| chrono::Duration::minutes(5))
    }
}

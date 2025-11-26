use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClientConfig {
    pub provider: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub authorization_url: String,
    pub token_url: String,
    pub redirect_uri: Option<String>,
    pub scopes: Vec<String>,
}

impl OAuthClientConfig {
    pub fn new(
        provider: impl Into<String>,
        client_id: impl Into<String>,
        authorization_url: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            client_id: client_id.into(),
            client_secret: None,
            authorization_url: authorization_url.into(),
            token_url: token_url.into(),
            redirect_uri: None,
            scopes: Vec::new(),
        }
    }

    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(redirect_uri.into());
        self
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }
}

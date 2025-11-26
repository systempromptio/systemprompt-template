use crate::models::oauth::dynamic_registration::DynamicRegistrationRequest;
use crate::repository::OAuthRepository;
use anyhow::{anyhow, Result};
use axum::{
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use systemprompt_core_system::DbPool;

#[derive(Debug, Clone)]
pub struct BrowserRedirectService {
    repo: OAuthRepository,
}

impl BrowserRedirectService {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            repo: OAuthRepository::new(db_pool),
        }
    }

    pub fn is_browser_request(headers: &HeaderMap) -> bool {
        headers
            .get("accept")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|accept| accept.contains("text/html") && !accept.starts_with("application/json"))
    }

    pub async fn create_oauth_redirect(
        &self,
        original_url: &str,
        server_base_url: &str,
    ) -> Result<impl IntoResponse> {
        let web_client = self.find_web_client(server_base_url).await?;

        let redirect_uri = format!("{server_base_url}/api/v1/core/oauth/callback");
        let encoded_redirect_uri = urlencoding::encode(&redirect_uri);
        let encoded_state = urlencoding::encode(original_url);

        let oauth_url = format!(
            "{}/api/v1/core/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
            server_base_url,
            web_client.client_id,
            encoded_redirect_uri,
            web_client.scopes.join("%20"),
            encoded_state
        );

        Ok(Redirect::to(&oauth_url))
    }

    async fn find_web_client(&self, server_base_url: &str) -> Result<WebClient> {
        let redirect_uri = format!("{server_base_url}/api/v1/core/oauth/callback");
        let clients = self.repo.list_clients().await?;

        // First, try to find existing client with the exact redirect URI we need
        for client in clients {
            if client.redirect_uris.contains(&redirect_uri)
                && (client.scopes.contains(&"admin".to_string())
                    || client.scopes.contains(&"user".to_string()))
                {
                    return Ok(WebClient {
                        client_id: client.client_id,
                        scopes: client.scopes,
                    });
                }
        }

        // No suitable client found, dynamically register one
        self.register_browser_client(server_base_url).await
    }

    async fn register_browser_client(&self, server_base_url: &str) -> Result<WebClient> {
        let redirect_uri = format!("{server_base_url}/api/v1/core/oauth/callback");

        let registration_request = DynamicRegistrationRequest {
            client_name: Some("SystemPrompt Browser Client".to_string()),
            redirect_uris: Some(vec![redirect_uri]),
            grant_types: Some(vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ]),
            response_types: Some(vec!["code".to_string()]),
            scope: Some("admin user".to_string()),
            token_endpoint_auth_method: Some("client_secret_basic".to_string()),
            client_uri: None,
            logo_uri: None,
            contacts: None,
            software_statement: None,
        };

        // Use the existing registration logic to create the client
        let client_id = format!("browser-client-{}", uuid::Uuid::new_v4().simple());
        let client_secret = uuid::Uuid::new_v4().to_string();
        let client_secret_hash = bcrypt::hash(&client_secret, bcrypt::DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash client secret: {e}"))?;

        let client_name = registration_request
            .get_client_name()
            .map_err(|e| anyhow!("Invalid client name: {e}"))?;
        let redirect_uris = registration_request
            .get_redirect_uris()
            .map_err(|e| anyhow!("Invalid redirect URIs: {e}"))?;
        let grant_types = registration_request
            .get_grant_types()
            .map_err(|e| anyhow!("Invalid grant types: {e}"))?;
        let response_types = registration_request
            .get_response_types()
            .map_err(|e| anyhow!("Invalid response types: {e}"))?;
        let scopes = vec!["admin".to_string(), "user".to_string()];
        let token_endpoint_auth_method = registration_request
            .get_token_endpoint_auth_method()
            .map_err(|e| anyhow!("Invalid token endpoint auth method: {e}"))?;

        self.repo
            .create_client(
                &client_id,
                &client_secret_hash,
                &client_name,
                &redirect_uris,
                Some(&grant_types),
                Some(&response_types),
                &scopes,
                Some(&token_endpoint_auth_method),
                None, // client_uri
                None, // logo_uri
                None, // contacts
            )
            .await
            .map_err(|e| anyhow!("Failed to register browser client: {e}"))?;

        Ok(WebClient { client_id, scopes })
    }
}

#[derive(Debug)]
struct WebClient {
    client_id: String,
    scopes: Vec<String>,
}

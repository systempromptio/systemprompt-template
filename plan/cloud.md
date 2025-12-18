# Cloud CLI Implementation Plan

**Target Repository**: `systemprompt-core`
**Location**: `src/cli/cloud/`

## Overview

Add a `cloud` subcommand group to the SystemPrompt CLI for deploying to SystemPrompt Cloud.
Uses ephemeral OAuth callback server for authentication and `.systemprompt/` for credential storage.

## User Experience

```bash
# Authentication
systemprompt cloud login                    # OAuth via browser
systemprompt cloud logout                   # Clear credentials

# Tenant management
systemprompt cloud setup                    # Create/link tenant
systemprompt cloud config                   # Show current config

# Deployment
systemprompt cloud deploy                   # Build + push + deploy
systemprompt cloud deploy --skip-build      # Deploy existing image
systemprompt cloud status                   # Check deployment status

# Via justfile (in template repos)
just cloud-login
just cloud-deploy
just cloud-status
```

---

## File Structure

```
systemprompt-core/src/cli/
├── cloud/
│   ├── mod.rs              # CloudCommands enum + execute()
│   ├── credentials.rs      # .systemprompt/ credential storage
│   ├── oauth.rs            # Ephemeral callback server
│   ├── api.rs              # systemprompt-db API client
│   ├── login.rs            # Login command handler
│   ├── logout.rs           # Logout command handler
│   ├── setup.rs            # Tenant setup handler
│   ├── deploy.rs           # Deploy command handler
│   ├── status.rs           # Status command handler
│   └── config.rs           # Config display handler
└── mod.rs                  # Add: pub mod cloud;
```

---

## 1. Commands Definition (`cloud/mod.rs`)

```rust
//! SystemPrompt Cloud deployment commands
//!
//! Provides authentication and deployment to SystemPrompt Cloud,
//! abstracting away the underlying Fly.io infrastructure.

use anyhow::Result;
use clap::Subcommand;

mod api;
mod config;
mod credentials;
mod deploy;
mod login;
mod logout;
mod oauth;
mod setup;
mod status;

pub use credentials::CloudCredentials;

/// Default API URL for SystemPrompt Cloud
pub const DEFAULT_API_URL: &str = "https://api.systemprompt.io";

#[derive(Subcommand)]
pub enum CloudCommands {
    /// Authenticate with SystemPrompt Cloud
    ///
    /// Opens browser for OAuth authentication. Creates a local callback
    /// server to receive the authentication token.
    Login {
        /// API base URL
        #[arg(long, env = "SYSTEMPROMPT_CLOUD_API_URL", default_value = DEFAULT_API_URL)]
        api_url: String,
    },

    /// Clear saved cloud credentials
    Logout,

    /// Link this project to a cloud tenant
    ///
    /// Creates a new tenant or links to an existing one.
    /// Requires prior authentication via `cloud login`.
    Setup {
        /// Tenant name (defaults to current directory name)
        #[arg(long)]
        name: Option<String>,

        /// Region for new tenants
        #[arg(long, default_value = "iad")]
        region: String,
    },

    /// Deploy to SystemPrompt Cloud
    ///
    /// Builds Docker image, pushes to registry, and triggers deployment.
    /// Requires prior authentication and tenant setup.
    Deploy {
        /// Skip cargo build and web asset compilation
        #[arg(long)]
        skip_build: bool,

        /// Skip Docker image push (deploy existing image)
        #[arg(long)]
        skip_push: bool,

        /// Custom image tag (default: deploy-{timestamp}-{git-sha})
        #[arg(long)]
        tag: Option<String>,
    },

    /// Check cloud deployment status
    Status,

    /// Show current cloud configuration
    Config,
}

pub async fn execute(cmd: CloudCommands) -> Result<()> {
    match cmd {
        CloudCommands::Login { api_url } => login::execute(&api_url).await,
        CloudCommands::Logout => logout::execute().await,
        CloudCommands::Setup { name, region } => setup::execute(name, &region).await,
        CloudCommands::Deploy {
            skip_build,
            skip_push,
            tag,
        } => deploy::execute(skip_build, skip_push, tag).await,
        CloudCommands::Status => status::execute().await,
        CloudCommands::Config => config::execute().await,
    }
}
```

---

## 2. Credentials Storage (`cloud/credentials.rs`)

```rust
//! Credential storage in .systemprompt/ directory

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CREDENTIALS_DIR: &str = ".systemprompt";
const CREDENTIALS_FILE: &str = "credentials.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCredentials {
    /// JWT token for API authentication
    pub api_token: String,

    /// API base URL
    pub api_url: String,

    /// Linked tenant ID (set after `cloud setup`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    /// Fly.io app name for the tenant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fly_app_name: Option<String>,

    /// Tenant's public hostname
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    /// When the user authenticated
    pub authenticated_at: DateTime<Utc>,

    /// User email (for display purposes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
}

impl CloudCredentials {
    /// Create new credentials after login
    pub fn new(api_token: String, api_url: String, user_email: Option<String>) -> Self {
        Self {
            api_token,
            api_url,
            tenant_id: None,
            fly_app_name: None,
            hostname: None,
            authenticated_at: Utc::now(),
            user_email,
        }
    }

    /// Path to credentials directory
    pub fn dir_path() -> PathBuf {
        PathBuf::from(CREDENTIALS_DIR)
    }

    /// Path to credentials file
    pub fn file_path() -> PathBuf {
        Self::dir_path().join(CREDENTIALS_FILE)
    }

    /// Check if credentials exist
    pub fn exists() -> bool {
        Self::file_path().exists()
    }

    /// Load credentials from disk
    pub fn load() -> Result<Self> {
        let path = Self::file_path();
        if !path.exists() {
            return Err(anyhow!(
                "Not logged in. Run 'systemprompt cloud login' first."
            ));
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| "Failed to parse credentials. Try logging in again.")
    }

    /// Save credentials to disk
    pub fn save(&self) -> Result<()> {
        let dir = Self::dir_path();
        fs::create_dir_all(&dir)?;

        // Create .gitignore in the directory
        let gitignore_path = dir.join(".gitignore");
        if !gitignore_path.exists() {
            fs::write(&gitignore_path, "*\n")?;
        }

        let path = Self::file_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Delete credentials
    pub fn delete() -> Result<()> {
        let path = Self::file_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Check if tenant is configured
    pub fn has_tenant(&self) -> bool {
        self.tenant_id.is_some() && self.fly_app_name.is_some()
    }

    /// Get tenant ID or error
    pub fn require_tenant(&self) -> Result<&str> {
        self.tenant_id.as_deref().ok_or_else(|| {
            anyhow!("No tenant configured. Run 'systemprompt cloud setup' first.")
        })
    }

    /// Update tenant info
    pub fn set_tenant(
        &mut self,
        tenant_id: String,
        fly_app_name: String,
        hostname: Option<String>,
    ) {
        self.tenant_id = Some(tenant_id);
        self.fly_app_name = Some(fly_app_name);
        self.hostname = hostname;
    }
}
```

---

## 3. OAuth Callback Server (`cloud/oauth.rs`)

```rust
//! Ephemeral OAuth callback server
//!
//! Starts a local HTTP server to receive the OAuth callback,
//! similar to `gh auth login`, `aws sso login`, etc.

use anyhow::{anyhow, Result};
use axum::{
    extract::Query,
    response::Html,
    routing::get,
    Router,
};
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

const CALLBACK_PORT: u16 = 8765;
const CALLBACK_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[derive(serde::Deserialize)]
struct CallbackParams {
    token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Run OAuth flow with ephemeral callback server
///
/// 1. Starts local server on localhost:8765
/// 2. Opens browser to OAuth URL
/// 3. Waits for callback with token
/// 4. Returns token or error
pub async fn run_oauth_flow(api_url: &str) -> Result<String> {
    let (tx, rx) = oneshot::channel::<Result<String>>();
    let tx = Arc::new(Mutex::new(Some(tx)));

    // Build callback handler
    let callback_handler = {
        let tx = tx.clone();
        move |Query(params): Query<CallbackParams>| {
            let tx = tx.clone();
            async move {
                let result = if let Some(error) = params.error {
                    let desc = params.error_description.unwrap_or_default();
                    Err(anyhow!("OAuth error: {} - {}", error, desc))
                } else if let Some(token) = params.token {
                    Ok(token)
                } else {
                    Err(anyhow!("No token received in callback"))
                };

                // Send result through channel
                if let Some(sender) = tx.lock().await.take() {
                    let is_success = result.is_ok();
                    let _ = sender.send(result);

                    if is_success {
                        Html(SUCCESS_HTML.to_string())
                    } else {
                        Html(ERROR_HTML.to_string())
                    }
                } else {
                    Html(ERROR_HTML.to_string())
                }
            }
        }
    };

    // Start server
    let app = Router::new().route("/callback", get(callback_handler));
    let addr = format!("127.0.0.1:{}", CALLBACK_PORT);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!(
        "{}",
        format!("Starting authentication server on http://{}", addr).dimmed()
    );

    // Build OAuth URL
    let redirect_uri = format!("http://127.0.0.1:{}/callback", CALLBACK_PORT);
    let auth_url = format!(
        "{}/auth/login?redirect_uri={}",
        api_url,
        urlencoding::encode(&redirect_uri)
    );

    // Open browser
    println!("{}", "Opening browser for authentication...".cyan());
    println!("{}", format!("URL: {}", auth_url).dimmed());

    if let Err(e) = open::that(&auth_url) {
        println!(
            "{}",
            format!("Could not open browser automatically: {}", e).yellow()
        );
        println!("Please open this URL manually:");
        println!("  {}", auth_url);
    }

    println!();
    println!("{}", "Waiting for authentication...".dimmed());
    println!(
        "{}",
        format!("(timeout in {} seconds)", CALLBACK_TIMEOUT_SECS).dimmed()
    );

    // Run server with timeout
    let server = axum::serve(listener, app);

    tokio::select! {
        result = rx => {
            result.map_err(|_| anyhow!("Authentication cancelled"))?
        }
        _ = server => {
            Err(anyhow!("Server stopped unexpectedly"))
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(CALLBACK_TIMEOUT_SECS)) => {
            Err(anyhow!("Authentication timed out after {} seconds", CALLBACK_TIMEOUT_SECS))
        }
    }
}

const SUCCESS_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Authentication Successful</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 40px;
            background: rgba(255,255,255,0.1);
            border-radius: 16px;
            backdrop-filter: blur(10px);
        }
        h1 { margin: 0 0 16px; font-size: 2em; }
        p { margin: 0; opacity: 0.9; }
        .check { font-size: 4em; margin-bottom: 16px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="check">✓</div>
        <h1>Authentication Successful</h1>
        <p>You can close this window and return to the terminal.</p>
    </div>
</body>
</html>
"#;

const ERROR_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Authentication Failed</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: #dc3545;
            color: white;
        }
        .container { text-align: center; padding: 40px; }
        h1 { margin: 0 0 16px; }
        .icon { font-size: 4em; margin-bottom: 16px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">✗</div>
        <h1>Authentication Failed</h1>
        <p>Please try again.</p>
    </div>
</body>
</html>
"#;
```

---

## 4. API Client (`cloud/api.rs`)

```rust
//! HTTP client for SystemPrompt Cloud API (systemprompt-db)

use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// API response wrapper
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

/// Error response
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

/// User info from /auth/me
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

/// Tenant info
#[derive(Debug, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub fly_app_name: Option<String>,
    pub fly_hostname: Option<String>,
    pub fly_status: Option<String>,
    pub status: String,
}

/// Create tenant response
#[derive(Debug, Deserialize)]
pub struct CreateTenantResponse {
    pub tenant: Tenant,
    pub credentials_info: String,
}

/// Tenant status
#[derive(Debug, Deserialize)]
pub struct TenantStatus {
    pub status: String,
    pub message: Option<String>,
    pub app_url: Option<String>,
    pub secrets_url: Option<String>,
}

/// Registry token response
#[derive(Debug, Deserialize)]
pub struct RegistryToken {
    pub registry: String,
    pub username: String,
    pub password: String,
    pub repository: String,
}

/// Deploy response
#[derive(Debug, Deserialize)]
pub struct DeployResponse {
    pub status: String,
    pub message: String,
    pub app_url: Option<String>,
    pub machine_id: Option<String>,
    pub deployed_at: String,
}

/// Create tenant request
#[derive(Debug, Serialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub region: String,
}

/// Deploy request
#[derive(Debug, Serialize)]
pub struct DeployRequest {
    pub image: String,
}

/// Paginated list response
#[derive(Debug, Deserialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
}

/// Cloud API client
pub struct CloudApiClient {
    client: Client,
    api_url: String,
    token: String,
}

impl CloudApiClient {
    pub fn new(api_url: &str, token: &str) -> Self {
        Self {
            client: Client::new(),
            api_url: api_url.to_string(),
            token: token.to_string(),
        }
    }

    /// Make authenticated GET request
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.api_url, path);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context("Failed to connect to API")?;

        self.handle_response(response).await
    }

    /// Make authenticated POST request
    async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.api_url, path);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(body)
            .send()
            .await
            .context("Failed to connect to API")?;

        self.handle_response(response).await
    }

    /// Handle API response
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(anyhow!(
                "Authentication failed. Please run 'systemprompt cloud login' again."
            ));
        }

        if !status.is_success() {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: ApiErrorDetail {
                    code: "unknown".to_string(),
                    message: format!("Request failed with status {}", status),
                },
            });
            return Err(anyhow!("{}: {}", error.error.code, error.error.message));
        }

        response
            .json()
            .await
            .context("Failed to parse API response")
    }

    /// Get current user info
    pub async fn get_user(&self) -> Result<UserInfo> {
        let response: ApiResponse<UserInfo> = self.get("/api/v1/auth/me").await?;
        Ok(response.data)
    }

    /// List user's tenants
    pub async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        let response: ListResponse<Tenant> = self.get("/api/v1/tenants").await?;
        Ok(response.data)
    }

    /// Create a free tenant
    pub async fn create_tenant(&self, name: &str, region: &str) -> Result<CreateTenantResponse> {
        let request = CreateTenantRequest {
            name: name.to_string(),
            region: region.to_string(),
        };
        let response: ApiResponse<CreateTenantResponse> =
            self.post("/api/v1/tenants/free", &request).await?;
        Ok(response.data)
    }

    /// Get tenant status
    pub async fn get_tenant_status(&self, tenant_id: &str) -> Result<TenantStatus> {
        let response: ApiResponse<TenantStatus> =
            self.get(&format!("/api/v1/tenants/{}/status", tenant_id)).await?;
        Ok(response.data)
    }

    /// Get registry token for pushing images
    pub async fn get_registry_token(&self, tenant_id: &str) -> Result<RegistryToken> {
        let response: ApiResponse<RegistryToken> = self
            .get(&format!("/api/v1/tenants/{}/registry-token", tenant_id))
            .await?;
        Ok(response.data)
    }

    /// Deploy an image to tenant
    pub async fn deploy(&self, tenant_id: &str, image: &str) -> Result<DeployResponse> {
        let request = DeployRequest {
            image: image.to_string(),
        };
        let response: ApiResponse<DeployResponse> = self
            .post(&format!("/api/v1/tenants/{}/deploy", tenant_id), &request)
            .await?;
        Ok(response.data)
    }
}
```

---

## 5. Login Command (`cloud/login.rs`)

```rust
//! Cloud login command handler

use anyhow::Result;
use colored::Colorize;

use super::api::CloudApiClient;
use super::credentials::CloudCredentials;
use super::oauth::run_oauth_flow;

pub async fn execute(api_url: &str) -> Result<()> {
    println!();
    println!("{}", "SystemPrompt Cloud Login".cyan().bold());
    println!("{}", "════════════════════════════════════════".dimmed());
    println!();

    // Check for existing credentials
    if CloudCredentials::exists() {
        let existing = CloudCredentials::load()?;
        if let Some(email) = &existing.user_email {
            println!(
                "{}",
                format!("Already logged in as: {}", email).yellow()
            );
            println!("{}", "Re-authenticating...".dimmed());
            println!();
        }
    }

    // Run OAuth flow
    let token = run_oauth_flow(api_url).await?;

    println!();
    println!("{}", "Verifying token...".dimmed());

    // Verify token and get user info
    let client = CloudApiClient::new(api_url, &token);
    let user = client.get_user().await?;

    // Save credentials
    let mut creds = CloudCredentials::new(token, api_url.to_string(), Some(user.email.clone()));

    // Preserve tenant info if re-authenticating
    if let Ok(existing) = CloudCredentials::load() {
        creds.tenant_id = existing.tenant_id;
        creds.fly_app_name = existing.fly_app_name;
        creds.hostname = existing.hostname;
    }

    creds.save()?;

    println!();
    println!("{}", "════════════════════════════════════════".green());
    println!(
        "{}",
        format!("  ✓ Logged in as: {}", user.email).green().bold()
    );
    println!("{}", "════════════════════════════════════════".green());
    println!();

    if creds.has_tenant() {
        println!(
            "{}",
            format!("  Tenant: {}", creds.fly_app_name.as_deref().unwrap_or("unknown")).dimmed()
        );
    } else {
        println!(
            "{}",
            "  Next step: Run 'systemprompt cloud setup' to link a tenant.".dimmed()
        );
    }
    println!();

    Ok(())
}
```

---

## 6. Logout Command (`cloud/logout.rs`)

```rust
//! Cloud logout command handler

use anyhow::Result;
use colored::Colorize;

use super::credentials::CloudCredentials;

pub async fn execute() -> Result<()> {
    if !CloudCredentials::exists() {
        println!("{}", "Not logged in.".yellow());
        return Ok(());
    }

    CloudCredentials::delete()?;

    println!();
    println!("{}", "✓ Logged out of SystemPrompt Cloud".green());
    println!();

    Ok(())
}
```

---

## 7. Setup Command (`cloud/setup.rs`)

```rust
//! Cloud tenant setup command handler

use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::{Select, Input};
use std::env;

use super::api::CloudApiClient;
use super::credentials::CloudCredentials;

pub async fn execute(name: Option<String>, region: &str) -> Result<()> {
    println!();
    println!("{}", "SystemPrompt Cloud Setup".cyan().bold());
    println!("{}", "════════════════════════════════════════".dimmed());
    println!();

    // Load credentials
    let mut creds = CloudCredentials::load()?;
    let client = CloudApiClient::new(&creds.api_url, &creds.api_token);

    // List existing tenants
    let tenants = client.list_tenants().await?;

    let tenant_id: String;
    let fly_app_name: String;
    let hostname: Option<String>;

    if tenants.is_empty() {
        // No tenants - create new one
        println!("{}", "No existing tenants found. Creating a new one...".dimmed());
        println!();

        let tenant_name = if let Some(n) = name {
            n
        } else {
            let default_name = env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
                .unwrap_or_else(|| "my-site".to_string())
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != '_', "_");

            Input::new()
                .with_prompt("Tenant name")
                .default(default_name)
                .interact_text()?
        };

        println!();
        println!("{}", format!("Creating tenant '{}' in region '{}'...", tenant_name, region).dimmed());

        let response = client.create_tenant(&tenant_name, region).await?;

        tenant_id = response.tenant.id;
        fly_app_name = response.tenant.fly_app_name.ok_or_else(|| {
            anyhow!("Tenant created but no Fly app assigned. Contact support.")
        })?;
        hostname = response.tenant.fly_hostname;

        println!();
        println!("{}", "✓ Tenant created successfully".green());
    } else {
        // Show existing tenants
        println!("{}", "Your existing tenants:".dimmed());
        println!();

        let options: Vec<String> = tenants
            .iter()
            .map(|t| {
                let host = t.fly_hostname.as_deref().unwrap_or("provisioning...");
                format!("{} → {}", t.name, host)
            })
            .chain(std::iter::once("+ Create new tenant".to_string()))
            .collect();

        let selection = Select::new()
            .with_prompt("Select a tenant")
            .items(&options)
            .default(0)
            .interact()?;

        if selection == tenants.len() {
            // Create new
            let tenant_name: String = Input::new()
                .with_prompt("New tenant name")
                .interact_text()?;

            println!();
            println!("{}", format!("Creating tenant '{}'...", tenant_name).dimmed());

            let response = client.create_tenant(&tenant_name, region).await?;

            tenant_id = response.tenant.id;
            fly_app_name = response.tenant.fly_app_name.ok_or_else(|| {
                anyhow!("Tenant created but no Fly app assigned")
            })?;
            hostname = response.tenant.fly_hostname;
        } else {
            // Use existing
            let selected = &tenants[selection];
            tenant_id = selected.id.clone();
            fly_app_name = selected.fly_app_name.clone().ok_or_else(|| {
                anyhow!("Selected tenant has no Fly app. Contact support.")
            })?;
            hostname = selected.fly_hostname.clone();
        }
    }

    // Save tenant info
    creds.set_tenant(tenant_id, fly_app_name.clone(), hostname.clone());
    creds.save()?;

    // Show summary
    println!();
    println!("{}", "════════════════════════════════════════".green());
    println!("{}", "  ✓ Tenant configured".green().bold());
    println!("{}", "════════════════════════════════════════".green());
    println!();
    println!("  App:  {}", fly_app_name);
    if let Some(h) = hostname {
        println!("  URL:  https://{}", h);
    }
    println!();
    println!(
        "{}",
        "  Next: Run 'systemprompt cloud deploy' to deploy your site.".dimmed()
    );
    println!();

    Ok(())
}
```

---

## 8. Deploy Command (`cloud/deploy.rs`)

```rust
//! Cloud deploy command handler

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use super::api::CloudApiClient;
use super::credentials::CloudCredentials;

pub async fn execute(skip_build: bool, skip_push: bool, custom_tag: Option<String>) -> Result<()> {
    println!();
    println!("{}", "SystemPrompt Cloud Deploy".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════".dimmed());
    println!();

    // Load credentials
    let creds = CloudCredentials::load()?;
    let tenant_id = creds.require_tenant()?;
    let fly_app_name = creds.fly_app_name.as_ref().ok_or_else(|| {
        anyhow!("No Fly app configured. Run 'systemprompt cloud setup' first.")
    })?;

    let client = CloudApiClient::new(&creds.api_url, &creds.api_token);
    let project_root = get_project_root()?;

    // Generate image tag
    let tag = custom_tag.unwrap_or_else(|| {
        let timestamp = chrono::Utc::now().timestamp();
        let git_sha = get_git_sha().unwrap_or_else(|| "unknown".to_string());
        format!("deploy-{}-{}", timestamp, git_sha)
    });

    let image = format!("registry.fly.io/{}:{}", fly_app_name, tag);

    println!("  Tenant: {}", fly_app_name);
    println!("  Image:  {}", image);
    println!();

    // Step 1: Build
    if !skip_build {
        println!("{}", "Step 1/4: Building...".yellow());
        build_release(&project_root)?;
        println!("{}", "         ✓ Build complete".green());
    } else {
        println!("{}", "Step 1/4: Build skipped".dimmed());
    }

    // Step 2: Docker build
    println!("{}", "Step 2/4: Building Docker image...".yellow());
    build_docker_image(&project_root, &image)?;
    println!("{}", "         ✓ Docker image built".green());

    // Step 3: Push
    if !skip_push {
        println!("{}", "Step 3/4: Pushing to registry...".yellow());

        // Get registry token
        let registry_token = client.get_registry_token(tenant_id).await?;

        // Docker login
        docker_login(&registry_token.registry, &registry_token.username, &registry_token.password)?;

        // Push image
        docker_push(&image)?;
        println!("{}", "         ✓ Image pushed".green());
    } else {
        println!("{}", "Step 3/4: Push skipped".dimmed());
    }

    // Step 4: Deploy
    println!("{}", "Step 4/4: Deploying...".yellow());
    let deploy_response = client.deploy(tenant_id, &image).await?;
    println!("{}", "         ✓ Deployed".green());

    // Summary
    println!();
    println!("{}", "═══════════════════════════════════════════════════".green().bold());
    println!("{}", "  ✓ Deployment Complete!".green().bold());
    println!("{}", "═══════════════════════════════════════════════════".green().bold());
    println!();
    println!("  Status: {}", deploy_response.status);
    if let Some(url) = deploy_response.app_url {
        println!("  URL:    {}", url.cyan());
    }
    println!();

    Ok(())
}

fn get_project_root() -> Result<PathBuf> {
    let current = env::current_dir()?;
    if current.join("infrastructure").exists() {
        Ok(current)
    } else {
        Err(anyhow!("Could not find project root. Run from project directory."))
    }
}

fn get_git_sha() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn build_release(project_root: &PathBuf) -> Result<()> {
    // Build Rust binary
    run_command(
        "cargo",
        &[
            "build",
            "--release",
            "--manifest-path=core/Cargo.toml",
            "--bin",
            "systemprompt",
        ],
        project_root,
    )?;

    // Build web assets
    run_command("npm", &["run", "build", "--prefix", "core/web"], project_root)?;

    // Stage artifacts
    let build_context = project_root.join("infrastructure/build-context/release");
    std::fs::create_dir_all(&build_context)?;
    std::fs::copy(
        project_root.join("core/target/release/systemprompt"),
        build_context.join("systemprompt"),
    )?;

    Ok(())
}

fn build_docker_image(project_root: &PathBuf, image: &str) -> Result<()> {
    run_command(
        "docker",
        &[
            "build",
            "-f",
            "infrastructure/docker/app.Dockerfile",
            "-t",
            image,
            ".",
        ],
        project_root,
    )
}

fn docker_login(registry: &str, username: &str, password: &str) -> Result<()> {
    let mut cmd = Command::new("docker");
    cmd.args(["login", registry, "-u", username, "--password-stdin"]);
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;
    if let Some(stdin) = child.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        stdin.write_all(password.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!("Docker login failed"));
    }

    Ok(())
}

fn docker_push(image: &str) -> Result<()> {
    run_command("docker", &["push", image], &env::current_dir()?)
}

fn run_command(cmd: &str, args: &[&str], dir: &PathBuf) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("Failed to run: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        return Err(anyhow!("Command failed: {} {}", cmd, args.join(" ")));
    }

    Ok(())
}
```

---

## 9. Status Command (`cloud/status.rs`)

```rust
//! Cloud status command handler

use anyhow::Result;
use colored::Colorize;

use super::api::CloudApiClient;
use super::credentials::CloudCredentials;

pub async fn execute() -> Result<()> {
    let creds = CloudCredentials::load()?;
    let tenant_id = creds.require_tenant()?;

    let client = CloudApiClient::new(&creds.api_url, &creds.api_token);
    let status = client.get_tenant_status(tenant_id).await?;

    println!();
    println!("{}", "SystemPrompt Cloud Status".cyan().bold());
    println!("{}", "════════════════════════════════════════".dimmed());
    println!();
    println!("  Tenant:  {}", creds.fly_app_name.as_deref().unwrap_or("unknown"));
    println!("  Status:  {}", colorize_status(&status.status));

    if let Some(msg) = status.message {
        println!("  Message: {}", msg.dimmed());
    }

    if let Some(url) = status.app_url {
        println!("  URL:     {}", url.cyan());
    }

    println!();

    Ok(())
}

fn colorize_status(status: &str) -> colored::ColoredString {
    match status {
        "ready" => status.green(),
        "provisioning" | "deploying" => status.yellow(),
        "failed" => status.red(),
        _ => status.normal(),
    }
}
```

---

## 10. Config Command (`cloud/config.rs`)

```rust
//! Cloud config display command handler

use anyhow::Result;
use colored::Colorize;

use super::credentials::CloudCredentials;

pub async fn execute() -> Result<()> {
    println!();
    println!("{}", "SystemPrompt Cloud Configuration".cyan().bold());
    println!("{}", "════════════════════════════════════════".dimmed());
    println!();

    if !CloudCredentials::exists() {
        println!("{}", "Not logged in.".yellow());
        println!("{}", "Run 'systemprompt cloud login' to authenticate.".dimmed());
        return Ok(());
    }

    let creds = CloudCredentials::load()?;

    println!("  API URL:    {}", creds.api_url);
    println!(
        "  User:       {}",
        creds.user_email.as_deref().unwrap_or("unknown")
    );
    println!(
        "  Logged in:  {}",
        creds.authenticated_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    println!();

    if let Some(tenant_id) = &creds.tenant_id {
        println!("  Tenant ID:  {}", tenant_id);
        println!(
            "  App Name:   {}",
            creds.fly_app_name.as_deref().unwrap_or("unknown")
        );
        if let Some(hostname) = &creds.hostname {
            println!("  Hostname:   {}", hostname);
            println!("  URL:        https://{}", hostname);
        }
    } else {
        println!("{}", "  No tenant configured.".yellow());
        println!("{}", "  Run 'systemprompt cloud setup' to link a tenant.".dimmed());
    }

    println!();
    println!("  Credentials: {}", CloudCredentials::file_path().display().to_string().dimmed());
    println!();

    Ok(())
}
```

---

## 11. Wire Up in `main.rs`

```rust
// In main.rs, add to Commands enum:
/// SystemPrompt Cloud deployment
#[command(subcommand)]
Cloud(cli::cloud::CloudCommands),

// In main.rs, add to match block:
Commands::Cloud(cmd) => {
    cli::cloud::execute(cmd).await?;
}
```

---

## 12. Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# Already present (likely):
anyhow = "1"
tokio = { version = "1", features = ["full"] }
axum = "0.7"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
colored = "2"

# New:
open = "5"              # For opening browser
urlencoding = "2"       # For URL encoding redirect_uri
dialoguer = "0.11"      # For interactive prompts in setup
```

---

## 13. Justfile Commands (for templates)

Templates should add these to their justfile:

```just
# ============================================================================
# CLOUD DEPLOYMENT
# ============================================================================

# Authenticate with SystemPrompt Cloud
cloud-login:
    ./core/target/debug/systemprompt cloud login

# Log out of SystemPrompt Cloud
cloud-logout:
    ./core/target/debug/systemprompt cloud logout

# Link this project to a cloud tenant
cloud-setup:
    ./core/target/debug/systemprompt cloud setup

# Deploy to SystemPrompt Cloud
cloud-deploy:
    ./core/target/debug/systemprompt cloud deploy

# Check cloud deployment status
cloud-status:
    ./core/target/debug/systemprompt cloud status

# Show cloud configuration
cloud-config:
    ./core/target/debug/systemprompt cloud config

# Full release: build and deploy to cloud
release: build-release
    ./core/target/release/systemprompt cloud deploy
```

---

## systemprompt-db Requirements

The API must support:

| Endpoint | Status | Notes |
|----------|--------|-------|
| `GET /auth/login?redirect_uri=...` | NEEDED | Redirect to OAuth with callback |
| `GET /api/v1/auth/me` | ✅ Exists | Returns user info |
| `GET /api/v1/tenants` | ✅ Exists | List user's tenants |
| `POST /api/v1/tenants/free` | ✅ Exists | Create free tenant |
| `GET /api/v1/tenants/{id}/status` | ✅ Exists | Get tenant status |
| `GET /api/v1/tenants/{id}/registry-token` | ✅ Added | Get Docker registry credentials |
| `POST /api/v1/tenants/{id}/deploy` | ✅ Added | Deploy image to tenant |

**OAuth redirect requirement:**
After successful OAuth, systemprompt-db must redirect to:
```
{redirect_uri}?token={jwt_token}
```

Example:
```
http://127.0.0.1:8765/callback?token=eyJhbGciOiJIUzI1NiIs...
```

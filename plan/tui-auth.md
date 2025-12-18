# TUI Cloud Authentication Integration

## Problem Statement

After running `just login sandbox`, the TUI (`systemprompt interactive`) does not use the cloud credentials. The cloud authentication and TUI are completely disconnected systems:

- **Cloud login** stores JWT to `~/.systemprompt/credentials.json`
- **TUI** uses local profiles from `services/profiles/*.profile.yml` and connects to local API

The user expectation is: after cloud login, the TUI should automatically authenticate with the cloud API using the stored credentials.

## Current Architecture

### Cloud Authentication Flow

**File:** `core/src/cli/cloud/login.rs`

1. User runs `systemprompt cloud login [sandbox|production]`
2. OAuth flow with GitHub/Google
3. Token stored to `~/.systemprompt/credentials.json`:

```json
{
  "api_token": "eyJhbGciOiJIUzI1NiIs...",
  "api_url": "https://api-sandbox.systemprompt.io",
  "tenant_id": "ctm_01jajcvkds81368bypdmsbsqje",
  "fly_app_name": null,
  "hostname": null,
  "authenticated_at": "2025-12-17T13:45:30.123456Z",
  "user_email": "user@example.com"
}
```

**File:** `core/src/cli/cloud/credentials.rs`

```rust
pub struct CloudCredentials {
    pub api_token: String,
    pub api_url: String,
    pub tenant_id: Option<String>,
    pub fly_app_name: Option<String>,
    pub hostname: Option<String>,
    pub authenticated_at: DateTime<Utc>,
    pub user_email: Option<String>,
}
```

### TUI Current Flow

**File:** `core/src/cli/interactive.rs`

1. Requires profile via `--profile` or `SYSTEMPROMPT_PROFILE` env var
2. Loads profile from `services/profiles/{name}.profile.yml`
3. Initializes `Config` from profile
4. Checks local API health at `profile.server.api_external_url`
5. Creates `AppContext` with local database connection
6. Launches `TuiApp` with local context

```rust
pub async fn execute(profile: Option<String>, _no_logs: bool, _no_sidebar: bool) -> Result<()> {
    let profile_name = resolve_profile_name(profile)?;
    let profile = load_profile(&profile_name)?;

    Config::init_from_profile(&profile)?;

    // Connects to LOCAL API
    let api_url = &profile.server.api_external_url;
    if !check_api_health(api_url).await {
        anyhow::bail!("API server is not reachable...");
    }

    // Uses LOCAL database
    let context = Arc::new(AppContext::new().await?);
    context.db_pool().test_connection().await?;

    let mut app = TuiApp::new(context, profile).await?;
    app.run().await?;
}
```

### TUI Components Using Authentication

**File:** `core/crates/modules/tui/src/app/mod.rs`

The TuiApp generates a local admin JWT token:

```rust
let admin_token = generate_admin_token(
    &Config::global().jwt_secret,
    &Config::global().jwt_issuer,
)?;
```

This token is used by:

1. **MessageSender** (`services/message_sender.rs`) - sends messages to agents
2. **ContextStreamSubscriber** (`services/context_stream_subscriber.rs`) - SSE streaming
3. **AgentDiscovery** - fetches agent list from API

All these use `Config::global().api_external_url` which comes from the local profile.

---

## Implementation Plan

### Phase 1: Fix Cloud Credentials Path Bug

**File:** `core/src/cli/cloud/credentials.rs`

**Problem:** Currently uses relative path `.systemprompt` instead of `$HOME/.systemprompt`.

**Solution:**

```rust
// Add to Cargo.toml
// dirs = "5.0"

use dirs::home_dir;

const CREDENTIALS_DIR: &str = ".systemprompt";
const CREDENTIALS_FILE: &str = "credentials.json";

impl CloudCredentials {
    /// Get the directory path for credentials (~/.systemprompt/)
    pub fn dir_path() -> PathBuf {
        home_dir()
            .unwrap_or_else(|| {
                // Fallback to current directory if home not found
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            })
            .join(CREDENTIALS_DIR)
    }

    /// Get the full file path for credentials (~/.systemprompt/credentials.json)
    pub fn file_path() -> PathBuf {
        Self::dir_path().join(CREDENTIALS_FILE)
    }
}
```

---

### Phase 2: Add Token Validation Methods

**File:** `core/src/cli/cloud/credentials.rs`

Add methods to validate token before use:

```rust
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::Deserialize;

impl CloudCredentials {
    /// Check if the stored token has expired
    ///
    /// Decodes JWT without verification to check exp claim.
    /// Returns true if token is expired or malformed.
    pub fn is_token_expired(&self) -> bool {
        #[derive(Deserialize)]
        struct Claims {
            exp: i64,
        }

        // JWT format: header.payload.signature
        let parts: Vec<&str> = self.api_token.split('.').collect();
        if parts.len() != 3 {
            return true; // Malformed JWT
        }

        // Decode payload (second part)
        let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
            Ok(p) => p,
            Err(_) => return true,
        };

        let claims: Claims = match serde_json::from_slice(&payload) {
            Ok(c) => c,
            Err(_) => return true,
        };

        let now = chrono::Utc::now().timestamp();
        claims.exp < now
    }

    /// Check if token will expire within the given duration
    pub fn expires_within(&self, duration: chrono::Duration) -> bool {
        #[derive(Deserialize)]
        struct Claims {
            exp: i64,
        }

        let parts: Vec<&str> = self.api_token.split('.').collect();
        if parts.len() != 3 {
            return true;
        }

        let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
            Ok(p) => p,
            Err(_) => return true,
        };

        let claims: Claims = match serde_json::from_slice(&payload) {
            Ok(c) => c,
            Err(_) => return true,
        };

        let now = chrono::Utc::now().timestamp();
        let threshold = now + duration.num_seconds();
        claims.exp < threshold
    }

    /// Load credentials and validate they are not expired
    ///
    /// Returns error if:
    /// - Credentials file doesn't exist
    /// - Credentials are malformed
    /// - Token has expired
    pub fn load_and_validate() -> Result<Self> {
        let creds = Self::load()?;

        if creds.is_token_expired() {
            return Err(anyhow!(
                "Cloud token has expired.\n\n\
                 Run 'systemprompt cloud login' to re-authenticate."
            ));
        }

        // Warn if expiring soon (within 1 hour)
        if creds.expires_within(chrono::Duration::hours(1)) {
            eprintln!(
                "Warning: Cloud token will expire soon. \
                 Consider running 'systemprompt cloud login' to refresh."
            );
        }

        Ok(creds)
    }

    /// Validate token against the cloud API
    ///
    /// Makes an API call to verify the token is still valid server-side.
    pub async fn validate_with_api(&self) -> Result<bool> {
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/api/v1/auth/me", self.api_url))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
```

---

### Phase 3: Create TuiMode Abstraction

**File:** `core/crates/modules/tui/src/mode.rs` (NEW FILE)

```rust
//! TUI operating modes
//!
//! The TUI can operate in two modes:
//! - Local: Connects to a local API server using profile configuration
//! - Cloud: Connects to SystemPrompt Cloud using OAuth credentials

use systemprompt_identifiers::JwtToken;
use systemprompt_models::Profile;

/// TUI operating mode
#[derive(Debug, Clone)]
pub enum TuiMode {
    /// Local development mode
    ///
    /// Uses a profile file for configuration and connects to local services.
    Local {
        /// Profile configuration
        profile: Profile,
    },

    /// Cloud mode
    ///
    /// Connects to SystemPrompt Cloud API using OAuth credentials.
    Cloud {
        /// Cloud API URL (e.g., https://api-sandbox.systemprompt.io)
        api_url: String,

        /// OAuth JWT token from cloud login
        token: JwtToken,

        /// Authenticated user's email
        user_email: Option<String>,

        /// Tenant ID for multi-tenant support
        tenant_id: Option<String>,
    },
}

impl TuiMode {
    /// Get the API URL for this mode
    pub fn api_url(&self) -> &str {
        match self {
            TuiMode::Local { profile } => &profile.server.api_external_url,
            TuiMode::Cloud { api_url, .. } => api_url,
        }
    }

    /// Get the authentication token for this mode
    pub fn token(&self) -> Option<&JwtToken> {
        match self {
            TuiMode::Local { .. } => None, // Local mode generates its own token
            TuiMode::Cloud { token, .. } => Some(token),
        }
    }

    /// Check if running in cloud mode
    pub fn is_cloud(&self) -> bool {
        matches!(self, TuiMode::Cloud { .. })
    }

    /// Check if running in local mode
    pub fn is_local(&self) -> bool {
        matches!(self, TuiMode::Local { .. })
    }

    /// Get display name for the mode
    pub fn display_name(&self) -> String {
        match self {
            TuiMode::Local { profile } => format!("Local ({})", profile.name),
            TuiMode::Cloud { api_url, .. } => {
                if api_url.contains("sandbox") {
                    "Cloud (Sandbox)".to_string()
                } else {
                    "Cloud (Production)".to_string()
                }
            }
        }
    }

    /// Get user identifier for display
    pub fn user_display(&self) -> Option<String> {
        match self {
            TuiMode::Local { .. } => Some("admin (local)".to_string()),
            TuiMode::Cloud { user_email, .. } => user_email.clone(),
        }
    }
}

impl std::fmt::Display for TuiMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
```

**Update:** `core/crates/modules/tui/src/lib.rs`

```rust
pub mod mode;
pub use mode::TuiMode;
```

---

### Phase 4: Modify Interactive Entry Point

**File:** `core/src/cli/interactive.rs`

Complete rewrite to support both modes:

```rust
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use systemprompt_core_system::{AppContext, Config};
use systemprompt_core_tui::{TuiApp, TuiMode};
use systemprompt_models::Profile;
use systemprompt_identifiers::JwtToken;

use crate::cli::cloud::credentials::CloudCredentials;

async fn check_api_health(api_url: &str) -> bool {
    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    else {
        return false;
    };

    let health_url = format!("{}/api/v1/health", api_url.trim_end_matches('/'));
    client.get(&health_url).send().await.is_ok()
}

fn resolve_profile_name(cli_profile: Option<String>) -> Result<String> {
    if let Some(profile) = cli_profile {
        return Ok(profile);
    }

    if let Ok(profile) = std::env::var("SYSTEMPROMPT_PROFILE") {
        return Ok(profile);
    }

    let services_path =
        std::env::var("SYSTEMPROMPT_SERVICES_PATH").unwrap_or_else(|_| "(not set)".to_string());

    let available = if services_path != "(not set)" {
        let profiles = Profile::list_available(Path::new(&services_path));
        if profiles.is_empty() {
            "  (none found)".to_string()
        } else {
            profiles
                .iter()
                .map(|p| format!("  - {}", p))
                .collect::<Vec<_>>()
                .join("\n")
        }
    } else {
        "  (SYSTEMPROMPT_SERVICES_PATH not set)".to_string()
    };

    anyhow::bail!(
        "No profile specified.\n\n\
         Please specify a profile using --profile <name> or SYSTEMPROMPT_PROFILE env var.\n\n\
         To generate a profile from your current .env:\n\
         \x20 systemprompt config profile generate --from-env --output dev\n\n\
         Available profiles in {}/profiles/:\n{}",
        services_path,
        available
    );
}

fn load_profile(profile_name: &str) -> Result<Profile> {
    let services_path = std::env::var("SYSTEMPROMPT_SERVICES_PATH").with_context(|| {
        "SYSTEMPROMPT_SERVICES_PATH environment variable must be set to load profiles"
    })?;

    Profile::load(Path::new(&services_path), profile_name).with_context(|| {
        format!(
            "Failed to load profile '{}' from {}/profiles/",
            profile_name, services_path
        )
    })
}

/// Determine the TUI mode based on CLI flags and available credentials
fn determine_mode(
    profile_arg: Option<String>,
    cloud_flag: bool,
    local_flag: bool,
) -> Result<TuiMode> {
    // Explicit --local flag forces local mode
    if local_flag {
        let profile_name = resolve_profile_name(profile_arg)?;
        let profile = load_profile(&profile_name)?;
        return Ok(TuiMode::Local { profile });
    }

    // Explicit --cloud flag requires cloud credentials
    if cloud_flag {
        let creds = CloudCredentials::load_and_validate().with_context(|| {
            "Cloud mode requested but no valid credentials found.\n\
             Run 'systemprompt cloud login' first."
        })?;

        return Ok(TuiMode::Cloud {
            api_url: creds.api_url,
            token: JwtToken::new(creds.api_token),
            user_email: creds.user_email,
            tenant_id: creds.tenant_id,
        });
    }

    // If profile is explicitly specified, use local mode
    if profile_arg.is_some() {
        let profile_name = resolve_profile_name(profile_arg)?;
        let profile = load_profile(&profile_name)?;
        return Ok(TuiMode::Local { profile });
    }

    // Auto-detect: prefer cloud if valid credentials exist
    if CloudCredentials::exists() {
        match CloudCredentials::load_and_validate() {
            Ok(creds) => {
                return Ok(TuiMode::Cloud {
                    api_url: creds.api_url,
                    token: JwtToken::new(creds.api_token),
                    user_email: creds.user_email,
                    tenant_id: creds.tenant_id,
                });
            }
            Err(e) => {
                // Cloud credentials exist but are invalid/expired
                // Log warning and fall through to local mode
                eprintln!("Warning: Cloud credentials invalid: {}", e);
                eprintln!("Falling back to local mode...\n");
            }
        }
    }

    // Fall back to local mode
    let profile_name = resolve_profile_name(None)?;
    let profile = load_profile(&profile_name)?;
    Ok(TuiMode::Local { profile })
}

/// Execute the interactive TUI
///
/// # Arguments
/// * `profile` - Optional profile name for local mode
/// * `cloud` - Force cloud mode (requires cloud login)
/// * `local` - Force local mode (requires profile)
/// * `no_logs` - Disable log panel
/// * `no_sidebar` - Disable sidebar
pub async fn execute(
    profile: Option<String>,
    cloud: bool,
    local: bool,
    no_logs: bool,
    no_sidebar: bool,
) -> Result<()> {
    dotenvy::dotenv().ok();

    // Determine operating mode
    let mode = determine_mode(profile, cloud, local)?;

    println!("Starting TUI in {} mode...", mode);
    if let Some(user) = mode.user_display() {
        println!("Authenticated as: {}", user);
    }
    println!();

    match &mode {
        TuiMode::Local { profile } => {
            execute_local_mode(profile.clone(), no_logs, no_sidebar).await
        }
        TuiMode::Cloud { api_url, token, user_email, tenant_id } => {
            execute_cloud_mode(
                api_url.clone(),
                token.clone(),
                user_email.clone(),
                tenant_id.clone(),
                no_logs,
                no_sidebar,
            ).await
        }
    }
}

/// Execute TUI in local mode
async fn execute_local_mode(
    profile: Profile,
    _no_logs: bool,
    _no_sidebar: bool,
) -> Result<()> {
    Config::init_from_profile(&profile)
        .context("Failed to initialize configuration from profile")?;

    let api_url = &profile.server.api_external_url;
    if !check_api_health(api_url).await {
        anyhow::bail!(
            "API server is not reachable at {}.\n\n\
             For local development, start the API first with: just start\n\
             Then run: just systemprompt",
            api_url
        );
    }

    let context = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    context
        .db_pool()
        .test_connection()
        .await
        .context("Failed to connect to database. Check profile database configuration.")?;

    let mut app = TuiApp::new_local(context, profile)
        .await
        .context("Failed to initialize TUI application")?;

    app.run().await.context("TUI application error")?;

    Ok(())
}

/// Execute TUI in cloud mode
async fn execute_cloud_mode(
    api_url: String,
    token: JwtToken,
    user_email: Option<String>,
    tenant_id: Option<String>,
    _no_logs: bool,
    _no_sidebar: bool,
) -> Result<()> {
    // Check cloud API is reachable
    if !check_api_health(&api_url).await {
        anyhow::bail!(
            "Cloud API is not reachable at {}.\n\n\
             Please check your internet connection.\n\
             If the problem persists, try 'systemprompt cloud login' again.",
            api_url
        );
    }

    // Validate token with API
    let creds = CloudCredentials {
        api_token: token.as_str().to_string(),
        api_url: api_url.clone(),
        tenant_id: tenant_id.clone(),
        fly_app_name: None,
        hostname: None,
        authenticated_at: chrono::Utc::now(),
        user_email: user_email.clone(),
    };

    if !creds.validate_with_api().await.unwrap_or(false) {
        anyhow::bail!(
            "Cloud token is no longer valid.\n\n\
             Run 'systemprompt cloud login' to re-authenticate."
        );
    }

    let mut app = TuiApp::new_cloud(api_url, token, user_email, tenant_id)
        .await
        .context("Failed to initialize TUI application in cloud mode")?;

    app.run().await.context("TUI application error")?;

    Ok(())
}
```

---

### Phase 5: Update CLI Command Definition

**File:** `core/src/cli/mod.rs`

Update the Interactive command to add new flags:

```rust
#[derive(Parser)]
pub enum Commands {
    // ... other commands ...

    /// Launch interactive TUI with AI chat interface
    Interactive {
        /// Profile name to load (for local mode)
        ///
        /// If not specified, uses SYSTEMPROMPT_PROFILE env var.
        /// Ignored when --cloud flag is used.
        #[arg(long, short = 'p')]
        profile: Option<String>,

        /// Connect to SystemPrompt Cloud
        ///
        /// Uses credentials from ~/.systemprompt/credentials.json.
        /// Run 'systemprompt cloud login' first.
        #[arg(long, conflicts_with = "local")]
        cloud: bool,

        /// Force local mode (requires profile)
        ///
        /// Ignores any cloud credentials and uses local profile.
        #[arg(long, conflicts_with = "cloud")]
        local: bool,

        /// Disable log panel
        #[arg(long)]
        no_logs: bool,

        /// Disable sidebar
        #[arg(long)]
        no_sidebar: bool,
    },
}
```

Update the command handler:

```rust
Commands::Interactive { profile, cloud, local, no_logs, no_sidebar } => {
    crate::cli::interactive::execute(profile, cloud, local, no_logs, no_sidebar).await
}
```

---

### Phase 6: Add Cloud Mode to TuiApp

**File:** `core/crates/modules/tui/src/app/mod.rs`

Add new constructor for cloud mode:

```rust
impl TuiApp {
    /// Create TUI in local mode with database access
    pub async fn new_local(context: Arc<AppContext>, profile: Profile) -> Result<Self> {
        // ... existing implementation renamed from new() ...
    }

    /// Create TUI in cloud mode (no local database)
    ///
    /// In cloud mode:
    /// - Uses cloud API for all operations
    /// - No local database connection
    /// - Uses OAuth token instead of local JWT
    /// - Tool execution is disabled (cloud handles it)
    pub async fn new_cloud(
        api_url: String,
        cloud_token: JwtToken,
        user_email: Option<String>,
        tenant_id: Option<String>,
    ) -> Result<Self> {
        let log_guard = init_file_logging();
        info!("TUI starting in CLOUD mode");
        info!("API URL: {}", api_url);
        if let Some(ref email) = user_email {
            info!("User: {}", email);
        }

        // Initialize terminal
        set_output_mode(OutputMode::Tui);
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        terminal.clear()?;

        let config = TuiConfig::default();
        let mut state = AppState::new(&config);

        // Show cloud mode in status
        state.set_mode_indicator(format!(
            "Cloud{}",
            if api_url.contains("sandbox") { " (Sandbox)" } else { "" }
        ));

        let (message_tx, message_rx) = mpsc::unbounded_channel();

        // Use cloud token directly (no local JWT generation)
        let admin_token = cloud_token.clone();

        // Create message sender with cloud API URL
        let message_sender = MessageSender::new_with_url(
            admin_token.clone(),
            message_tx.clone(),
            api_url.clone(),
        );

        // Fetch or create context from cloud API
        let context_id = cloud_api::fetch_or_create_context(&api_url, &admin_token)
            .await
            .context("Failed to get context from cloud API")?;

        let current_context_id = Arc::new(RwLock::new(context_id.clone()));

        // Initialize log service (local file logging still works)
        let log_service = Arc::new(LogService::new());

        // Create SSE subscriber for cloud API
        let stream_subscriber = ContextStreamSubscriber::new_with_url(
            message_tx.clone(),
            admin_token.clone(),
            current_context_id.clone(),
            log_service.clone(),
            api_url.clone(),
        );

        // Fetch agents from cloud API
        let agents = cloud_api::fetch_agents(&api_url, &admin_token)
            .await
            .context("Failed to fetch agents from cloud API")?;

        state.agents.set_agents_with_cards(agents.clone());

        let current_agent_name = agents
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "assistant".to_string());

        // Tool registry (empty in cloud mode - tools run server-side)
        let tool_registry = ToolRegistry::new();

        Ok(Self {
            terminal,
            state,
            config,
            message_tx,
            message_rx,
            context: None, // No local AppContext in cloud mode
            tool_registry,
            tool_executor: None, // No local tool execution
            message_sender,
            admin_token,
            current_agent_name,
            current_context_id,
            task_repository: None, // No local DB
            artifact_repository: None, // No local DB
            log_service,
            log_guard,
            mode: TuiMode::Cloud {
                api_url,
                token: cloud_token,
                user_email,
                tenant_id,
            },
        })
    }
}
```

---

### Phase 7: Update Service Components

**File:** `core/crates/modules/tui/src/services/message_sender.rs`

```rust
impl MessageSender {
    /// Create with explicit API URL (for cloud mode)
    pub fn new_with_url(
        admin_jwt: JwtToken,
        message_tx: UnboundedSender<Message>,
        api_base_url: String,
    ) -> Self {
        Self {
            client: Client::new(),
            api_base_url: api_base_url.trim_end_matches('/').to_string(),
            admin_jwt,
            message_tx,
        }
    }

    /// Create using global config (for local mode)
    pub fn new(admin_jwt: JwtToken, message_tx: UnboundedSender<Message>) -> Self {
        Self::new_with_url(
            admin_jwt,
            message_tx,
            Config::global().api_external_url.clone(),
        )
    }
}
```

**File:** `core/crates/modules/tui/src/services/context_stream_subscriber.rs`

```rust
impl ContextStreamSubscriber {
    /// Create with explicit API URL (for cloud mode)
    pub fn new_with_url(
        message_tx: UnboundedSender<Message>,
        auth_token: JwtToken,
        current_context_id: Arc<RwLock<ContextId>>,
        log_service: Arc<LogService>,
        api_base_url: String,
    ) -> Self {
        Self {
            message_tx,
            auth_token,
            api_base_url: api_base_url.trim_end_matches('/').to_string(),
            current_context_id,
            log_service,
        }
    }

    /// Create using global config (for local mode)
    pub fn new(
        message_tx: UnboundedSender<Message>,
        auth_token: JwtToken,
        current_context_id: Arc<RwLock<ContextId>>,
        log_service: Arc<LogService>,
    ) -> Self {
        Self::new_with_url(
            message_tx,
            auth_token,
            current_context_id,
            log_service,
            Config::global().api_external_url.clone(),
        )
    }
}
```

---

### Phase 8: Add Cloud API Helpers

**File:** `core/crates/modules/tui/src/services/cloud_api.rs` (NEW FILE)

```rust
//! Cloud API helpers for TUI cloud mode
//!
//! These functions interact with the SystemPrompt Cloud API
//! for operations that would normally use the local database.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use systemprompt_identifiers::{ContextId, JwtToken};

use crate::app::AgentCard;

/// Fetch existing context or create a new one
pub async fn fetch_or_create_context(
    api_url: &str,
    token: &JwtToken,
) -> Result<ContextId> {
    let client = Client::new();
    let api_url = api_url.trim_end_matches('/');

    // Try to get existing contexts
    let response = client
        .get(format!("{}/api/v1/contexts", api_url))
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .query(&[("limit", "1"), ("sort", "created_at:desc")])
        .send()
        .await
        .context("Failed to fetch contexts from cloud API")?;

    if response.status().is_success() {
        #[derive(Deserialize)]
        struct ContextList {
            data: Vec<ContextItem>,
        }
        #[derive(Deserialize)]
        struct ContextItem {
            context_id: String,
        }

        let list: ContextList = response
            .json()
            .await
            .context("Failed to parse contexts response")?;

        if let Some(ctx) = list.data.first() {
            return Ok(ContextId::new(&ctx.context_id));
        }
    }

    // No existing context, create a new one
    create_context(api_url, token).await
}

/// Create a new context in the cloud
pub async fn create_context(api_url: &str, token: &JwtToken) -> Result<ContextId> {
    let client = Client::new();
    let api_url = api_url.trim_end_matches('/');

    #[derive(Serialize)]
    struct CreateContextRequest {
        name: String,
    }

    let request = CreateContextRequest {
        name: format!("TUI Session {}", chrono::Utc::now().format("%Y-%m-%d %H:%M")),
    };

    let response = client
        .post(format!("{}/api/v1/contexts", api_url))
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to create context in cloud API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to create context: {} - {}", status, body);
    }

    #[derive(Deserialize)]
    struct CreateResponse {
        data: ContextData,
    }
    #[derive(Deserialize)]
    struct ContextData {
        context_id: String,
    }

    let created: CreateResponse = response
        .json()
        .await
        .context("Failed to parse create context response")?;

    Ok(ContextId::new(&created.data.context_id))
}

/// Fetch available agents from cloud API
pub async fn fetch_agents(api_url: &str, token: &JwtToken) -> Result<Vec<AgentCard>> {
    let client = Client::new();
    let api_url = api_url.trim_end_matches('/');

    let response = client
        .get(format!("{}/api/v1/agents", api_url))
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .send()
        .await
        .context("Failed to fetch agents from cloud API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to fetch agents: {} - {}", status, body);
    }

    #[derive(Deserialize)]
    struct AgentList {
        data: Vec<AgentData>,
    }
    #[derive(Deserialize)]
    struct AgentData {
        name: String,
        display_name: Option<String>,
        description: Option<String>,
        #[serde(default)]
        capabilities: Vec<String>,
    }

    let list: AgentList = response
        .json()
        .await
        .context("Failed to parse agents response")?;

    let agents = list
        .data
        .into_iter()
        .map(|a| AgentCard {
            name: a.name.clone(),
            display_name: a.display_name.unwrap_or_else(|| a.name.clone()),
            description: a.description.unwrap_or_default(),
            capabilities: a.capabilities,
        })
        .collect();

    Ok(agents)
}

/// Verify the token is still valid with the cloud API
pub async fn verify_token(api_url: &str, token: &JwtToken) -> Result<bool> {
    let client = Client::new();
    let api_url = api_url.trim_end_matches('/');

    let response = client
        .get(format!("{}/api/v1/auth/me", api_url))
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    Ok(response.status().is_success())
}
```

---

## Justfile Updates

**File:** `/var/www/html/systemprompt-template/justfile`

Update the `systemprompt` recipe:

```just
# Open interactive TUI
# Modes:
#   - Default: Auto-detects cloud credentials, falls back to local profile
#   - Cloud: just systemprompt --cloud (requires: just login)
#   - Local: just systemprompt --local --profile=dev
systemprompt *args="":
    ./core/target/debug/systemprompt interactive {{args}}
```

---

## Usage Examples

### Cloud Mode (after login)

```bash
# Login to sandbox
just login sandbox

# Start TUI (auto-detects cloud credentials)
just systemprompt

# Explicitly request cloud mode
just systemprompt --cloud
```

### Local Mode

```bash
# Start local services first
just start

# Use specific profile
just systemprompt --profile local

# Force local mode even if cloud credentials exist
just systemprompt --local --profile dev
```

### Mixed Development

```bash
# Work with sandbox cloud
just login sandbox
just systemprompt --cloud

# Switch to local development
just systemprompt --local --profile local

# Switch to production cloud
just login production
just systemprompt --cloud
```

---

## Testing Plan

### Unit Tests

1. `CloudCredentials::is_token_expired()` - test with valid/expired tokens
2. `CloudCredentials::load_and_validate()` - test error cases
3. `determine_mode()` - test flag combinations and auto-detection

### Integration Tests

1. Cloud mode with valid credentials → connects to cloud API
2. Cloud mode with expired credentials → prompts re-login
3. Local mode with profile → connects to local API
4. Auto-detect with cloud credentials → uses cloud mode
5. Auto-detect without credentials → falls back to local

### Manual Testing

1. `just login sandbox` then `just systemprompt` → cloud mode works
2. `just systemprompt --cloud` without login → clear error message
3. `just systemprompt --local --profile local` → local mode works
4. Cloud token expires → graceful error with re-login prompt

---

## Migration Notes

### Breaking Changes

None - existing behavior is preserved:
- `systemprompt interactive --profile <name>` continues to work
- Local mode is still available

### New Behavior

- If cloud credentials exist and are valid, TUI defaults to cloud mode
- Use `--local` flag to force local mode when cloud credentials exist

### Backwards Compatibility

- Profiles still work for local development
- Environment variables (`SYSTEMPROMPT_PROFILE`) still respected
- No changes to API contracts

---
title: "Cloud Credentials"
description: "Configure cloud authentication tokens and API validation."
keywords:
  - credentials
  - cloud
  - authentication
  - token
  - api
  - oauth
category: config
---

# Cloud Credentials

Configure cloud authentication tokens and API validation.

> **Help**: `{ "command": "cloud auth whoami" }` via `systemprompt_help`
> **Requires**: Profile configured -> See [Profiles Playbook](../profiles/index.md)

CloudCredentials authenticate CLI and API requests to SystemPrompt Cloud. Optional for local-only deployments.

---

## CloudCredentials Struct

**Source**: `crates/infra/cloud/src/credentials.rs:13-25`

```rust
pub struct CloudCredentials {
    #[validate(length(min = 1))]
    pub api_token: String,                     // Line 16 - JWT token
    #[validate(url)]
    pub api_url: String,                       // Line 19 - Cloud API endpoint
    pub authenticated_at: DateTime<Utc>,       // Line 21 - Login timestamp
    #[validate(email)]
    pub user_email: String,                    // Line 24 - User email
}
```

### Field Details

| Field | Type | Validation | Description |
|-------|------|------------|-------------|
| `api_token` | String | Non-empty | JWT token from OAuth login |
| `api_url` | String | Valid URL | Cloud API endpoint (default: `https://api.systemprompt.io`) |
| `authenticated_at` | DateTime | - | When the token was issued |
| `user_email` | String | Valid email | User's email address |

---

## CredentialsBootstrap

**Source**: `crates/infra/cloud/src/credentials_bootstrap.rs`

### Static Storage

```rust
static CREDENTIALS: OnceLock<Option<CloudCredentials>> = OnceLock::new();
```

Note: Stores `Option<CloudCredentials>` because cloud credentials are optional.

### Initialization (Async)

```rust
pub async fn init() -> Result<Option<&'static CloudCredentials>> {
    if std::env::var("FLY_APP_NAME").is_ok() {
        // Fly.io container: load from environment
        let creds = Self::load_from_env()?;
        Self::validate_with_api(&creds).await?;
        Ok(CREDENTIALS.get_or_init(|| Some(creds)).as_ref())
    } else {
        // Local: load from credentials.json
        let path = Self::get_credentials_path()?;
        let creds = CloudCredentials::load_from_path(&path)?;
        Self::check_expiration(&creds)?;
        Self::validate_with_api(&creds).await?;
        Ok(CREDENTIALS.get_or_init(|| Some(creds)).as_ref())
    }
}
```

### Error Types

```rust
pub enum CredentialsBootstrapError {
    NotInitialized,              // Not yet loaded
    AlreadyInitialized,          // Already initialized
    NotAvailable,                // Cloud not configured
    FileNotFound { path },       // credentials.json missing
    InvalidCredentials { message }, // Parse/validation error
    TokenExpired,                // Token older than 24 hours
    ApiValidationFailed { message }, // Cloud API rejected token
}
```

---

## Token Expiration

**Source**: `crates/infra/cloud/src/credentials.rs:140-149`

Cloud tokens expire 24 hours after authentication.

### Expiration Check

```rust
impl CloudCredentials {
    pub fn expires_within(&self, duration: Duration) -> bool {
        let expires_at = self.authenticated_at + Duration::hours(24);
        Utc::now() + duration >= expires_at
    }

    pub fn is_expired(&self) -> bool {
        self.expires_within(Duration::zero())
    }
}
```

### Expiration Handling

| Condition | Behavior |
|-----------|----------|
| Expired | Error: `TokenExpired` |
| Expires within 1 hour | Warning logged |
| Valid | Continue normally |

---

## File-Based Loading

### credentials.json Format

```json
{
  "api_token": "sp_live_eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "api_url": "https://api.systemprompt.io",
  "authenticated_at": "2026-02-01T10:00:00Z",
  "user_email": "user@example.com"
}
```

### File Location

The credentials file path comes from the profile:

```yaml
# profile.yaml
cloud:
  credentials_path: "../../credentials.json"  # Relative to profile dir
```

Default location: `.systemprompt/credentials.json`

### Loading Code

```rust
impl CloudCredentials {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|_| FileNotFound { path })?;

        let creds: CloudCredentials = serde_json::from_str(&content)
            .map_err(|e| InvalidCredentials { message: e.to_string() })?;

        creds.validate()?;  // Uses validator crate
        Ok(creds)
    }
}
```

---

## Environment-Based Loading

When running in a Fly.io container, credentials load from environment variables.

### Environment Variables

| Env Variable | Required | Default |
|--------------|----------|---------|
| `SYSTEMPROMPT_API_TOKEN` | Yes | - |
| `SYSTEMPROMPT_USER_EMAIL` | Yes | - |
| `SYSTEMPROMPT_API_URL` | No | `https://api.systemprompt.io` |

### Loading Code

**Source**: `crates/infra/cloud/src/credentials_bootstrap.rs:96-116`

```rust
fn load_from_env() -> Result<CloudCredentials> {
    let api_token = std::env::var("SYSTEMPROMPT_API_TOKEN")
        .map_err(|_| InvalidCredentials {
            message: "SYSTEMPROMPT_API_TOKEN not set".to_string()
        })?;

    let user_email = std::env::var("SYSTEMPROMPT_USER_EMAIL")
        .map_err(|_| InvalidCredentials {
            message: "SYSTEMPROMPT_USER_EMAIL not set".to_string()
        })?;

    let api_url = std::env::var("SYSTEMPROMPT_API_URL")
        .unwrap_or_else(|_| "https://api.systemprompt.io".to_string());

    Ok(CloudCredentials {
        api_token,
        api_url,
        authenticated_at: Utc::now(),  // Set to now for env-based
        user_email,
    })
}
```

---

## API Validation

After loading, credentials are validated against the Cloud API.

**Source**: `crates/infra/cloud/src/credentials_bootstrap.rs:81-90`

```rust
async fn validate_with_api(credentials: &CloudCredentials) -> Result<()> {
    let client = CloudApiClient::new(
        &credentials.api_url,
        &credentials.api_token
    );

    client.get_user().await
        .map_err(|e| ApiValidationFailed {
            message: format!("API validation failed: {}", e)
        })?;

    Ok(())
}
```

### What is Validated

- Token is not revoked
- Token has correct permissions
- API endpoint is reachable

---

## Fly.io Container Detection

```rust
fn is_fly_container() -> bool {
    std::env::var("FLY_APP_NAME").is_ok()
}
```

When `FLY_APP_NAME` is set:
1. Skip file-based loading
2. Load from environment variables
3. `authenticated_at` is set to current time

---

## Access Methods

### Get Credentials

```rust
// Get optional credentials (Ok(None) if not configured)
CredentialsBootstrap::get() -> Result<Option<&'static CloudCredentials>>

// Require credentials (error if not configured)
CredentialsBootstrap::require() -> Result<&'static CloudCredentials>

// Check if initialized
CredentialsBootstrap::is_initialized() -> bool

// Idempotent initialization
CredentialsBootstrap::try_init() -> Result<Option<&'static CloudCredentials>>

// Reload from file (ignores OnceLock)
CredentialsBootstrap::reload() -> Result<CloudCredentials>
```

### Static Expiration Check

```rust
// Check if credentials expire within duration
CredentialsBootstrap::expires_within(duration: Duration) -> bool
```

---

## OAuth Login Flow

Credentials are created through OAuth login:

```bash
# Terminal-based login
just login

# Or direct CLI
systemprompt cloud auth login
```

### Login Process

1. Open browser to OAuth provider (GitHub/Google)
2. User authenticates with provider
3. Callback to local server with code
4. Exchange code for SystemPrompt API token
5. Save to `.systemprompt/credentials.json`

### What Gets Saved

```json
{
  "api_token": "sp_live_...",
  "api_url": "https://api.systemprompt.io",
  "authenticated_at": "2026-02-01T10:00:00Z",
  "user_email": "user@example.com"
}
```

---

## Profile Configuration

Configure cloud credentials path in profile:

```yaml
# .systemprompt/profiles/local/profile.yaml
cloud:
  credentials_path: "../../credentials.json"
  tenants_path: "../../tenants.json"
  tenant_id: local_19bff27604c
  validation: strict  # strict | warn | skip
```

### Validation Modes

| Mode | Missing File | Invalid Token | API Failure |
|------|--------------|---------------|-------------|
| `strict` | Error | Error | Error |
| `warn` | Warning | Warning | Warning |
| `skip` | Silent | Silent | Silent |

---

## Troubleshooting

**"Credentials not initialized"**
- Run `just login` to authenticate
- Check credentials.json exists

**"Token expired"**
- Re-authenticate: `just login`
- Tokens expire after 24 hours

**"API validation failed"**
- Check network connectivity
- Verify api_url is correct
- Token may be revoked

**"File not found"**
- Verify path in profile's `cloud.credentials_path`
- Path is relative to profile directory

**"Invalid credentials"**
- Check JSON syntax
- Verify all required fields present
- Validate email format

---

## Security Best Practices

1. **Never commit**: Add `.systemprompt/credentials.json` to `.gitignore`
2. **File permissions**: Set to `0600` (owner read/write only)
3. **Token rotation**: Re-login periodically
4. **Separate credentials**: Don't share between environments
5. **Revoke on compromise**: Contact support to revoke tokens

---

## Quick Reference

| Task | Command / Location |
|------|-------------------|
| Login | `just login` |
| Check auth | `systemprompt cloud auth whoami` |
| Logout | `just logout` |
| View credentials | `.systemprompt/credentials.json` |
| Set in container | `SYSTEMPROMPT_API_TOKEN`, `SYSTEMPROMPT_USER_EMAIL` |
| API endpoint | `SYSTEMPROMPT_API_URL` (default: `https://api.systemprompt.io`) |

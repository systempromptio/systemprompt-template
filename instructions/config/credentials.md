# Cloud Credentials

Cloud credentials enable sync and deploy operations with SystemPrompt cloud infrastructure.

---

## Initialization

Credentials are initialized during application bootstrap:

```rust
use systemprompt::credentials::CredentialsBootstrap;

CredentialsBootstrap::init().context("Cloud credentials initialization failed")?;
```

This loads credentials from the path specified in the profile's cloud configuration.

---

## Profile Configuration

```yaml
cloud:
  validation_mode: strict
  credentials_path: ~/.systemprompt/credentials.json
```

| Field | Type | Description |
|-------|------|-------------|
| `validation_mode` | string | How to handle credential validation |
| `credentials_path` | string | Path to credentials JSON file |

---

## Validation Modes

| Mode | Behavior |
|------|----------|
| `strict` | Fail at startup if credentials invalid or missing |
| `warn` | Log warning, continue without credentials |
| `skip` | Skip credential validation entirely |

---

## Credentials File Structure

```json
{
  "tenant_id": "my-tenant",
  "api_token": "eyJ...",
  "api_url": "https://api.systemprompt.io",
  "token_expiry": "2025-12-31T23:59:59Z"
}
```

---

## API Reference

| Method | Signature | Use Case |
|--------|-----------|----------|
| `init()` | `fn init() -> Result<()>` | Called once at startup |
| `get()` | `fn get() -> Result<Option<CloudCredentials>>` | Optional credential check |
| `require()` | `fn require() -> Result<CloudCredentials>` | Fails if credentials missing |
| `is_initialized()` | `fn is_initialized() -> bool` | Check if init was called |

---

## Usage Patterns

### Optional Check (status tool)

```rust
let creds = CredentialsBootstrap::get().ok().flatten();
let is_configured = creds
    .as_ref()
    .map(|c| c.tenant_id.is_some() && !c.api_token.is_empty())
    .unwrap_or(false);
```

### Required Access (sync/deploy)

```rust
let creds = CredentialsBootstrap::require()
    .context("Cloud credentials required for sync")?;

let tenant_id = creds.tenant_id.as_ref()
    .ok_or_else(|| anyhow::anyhow!("No tenant configured"))?;
```

### Token Expiry Check

```rust
if creds.is_token_expired() {
    return Err(anyhow::anyhow!("Token expired"));
}
```

---

## Error Handling

Tools use `map_credential_error()` for user-friendly messages:

```rust
pub fn map_credential_error(err: &anyhow::Error) -> String {
    let msg = err.to_string();
    if msg.contains("No tenant configured") {
        "Run 'systemprompt cloud setup'".to_string()
    } else if msg.contains("Cloud credentials not found") {
        "Run 'systemprompt cloud login'".to_string()
    } else if msg.contains("Token expired") {
        "Run 'systemprompt cloud login'".to_string()
    } else {
        msg
    }
}
```

---

## CloudCredentials Fields

| Field | Type | Description |
|-------|------|-------------|
| `tenant_id` | `Option<String>` | Cloud tenant identifier |
| `api_token` | `String` | Authentication token |
| `api_url` | `String` | Cloud API endpoint |

---

## Setting Up Credentials

1. Run `systemprompt cloud setup` to configure tenant
2. Run `systemprompt cloud login` to authenticate
3. Credentials are stored at the path specified in profile

---

## See Also

- [profiles.md](./profiles.md) - Profile system documentation
- [../../plan/config.md](../../plan/config.md) - Implementation reference

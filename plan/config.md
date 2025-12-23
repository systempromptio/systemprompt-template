# Credentials & Profile Integration

This document describes how credentials and profiles are integrated in the MCP infrastructure server.

---

## Bootstrap Sequence

```
SYSTEMPROMPT_PROFILE=/path/to/profile.yml
         │
         ▼
ProfileBootstrap::init()
         │ (loads and validates profile)
         ▼
CredentialsBootstrap::init()
         │ (loads cloud credentials, fail-fast if strict mode)
         ▼
Config::init()
         │ (builds config from profile)
         ▼
AppContext::new()
         │ (creates db pool, registries)
         ▼
init_logging(db_pool)
         │ (database-persisted logging)
         ▼
Server Ready
```

---

## Main.rs Implementation

```rust
use systemprompt::credentials::CredentialsBootstrap;
use systemprompt::profile::ProfileBootstrap;
use systemprompt::models::Config;
use systemprompt::system::AppContext;
use systemprompt::logging;

#[tokio::main]
async fn main() -> Result<()> {
    ProfileBootstrap::init().context("Profile initialization failed")?;
    CredentialsBootstrap::init().context("Cloud credentials initialization failed")?;
    Config::init().context("Failed to initialize configuration")?;

    let ctx = Arc::new(AppContext::new().await?);
    logging::init_logging(ctx.db_pool().clone());

    let config = Config::global();
    let port = config.port;
    // ...
}
```

---

## Cloud Tools Usage

| Tool | Credential Usage | Pattern |
|------|-----------------|---------|
| status | Optional check | `CredentialsBootstrap::get()` |
| sync | Required | `CredentialsBootstrap::require()` |
| deploy | Required | `CredentialsBootstrap::require()` |
| config | Not required | - |
| export | Not required | - |

---

## CredentialsBootstrap API

| Method | Returns | Use Case |
|--------|---------|----------|
| `init()` | `Result<()>` | Called once at startup |
| `get()` | `Result<Option<CloudCredentials>>` | Optional credential check |
| `require()` | `Result<CloudCredentials>` | Fails if credentials missing |
| `is_initialized()` | `bool` | Check if init was called |

---

## Credential States

| State | Description | User Action |
|-------|-------------|-------------|
| authenticated | Valid token present | None |
| expired | Token expired | `systemprompt cloud login` |
| not authenticated | No credentials | `systemprompt cloud login` |
| not configured | No tenant | `systemprompt cloud setup` |

---

## Error Handling

Cloud tools use `map_credential_error()` in `src/tools/mod.rs`:

| Error Contains | User Message |
|---------------|--------------|
| "No tenant configured" | Run 'systemprompt cloud setup' |
| "Cloud credentials not found" | Run 'systemprompt cloud login' |
| "Token expired" | Run 'systemprompt cloud login' |

---

## SyncService Credential Usage

```rust
fn build_core_config(direction: CoreSyncDirection, dry_run: bool) -> Result<CoreSyncConfig> {
    let config = Config::global();
    let creds = CredentialsBootstrap::require()
        .context("Cloud credentials required for sync")?;

    let tenant_id = creds.tenant_id.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No tenant configured"))?;

    CoreSyncConfig::builder(
        tenant_id,
        &creds.api_url,
        &creds.api_token,
        &config.services_path,
    )
    .with_direction(direction)
    .with_dry_run(dry_run)
    .build()
}
```

---

## Status Tool Display

The status tool shows credential state:

```rust
let auth_status = match CredentialsBootstrap::get() {
    Ok(Some(creds)) => {
        if creds.is_token_expired() {
            "expired (run 'systemprompt cloud login')"
        } else {
            "authenticated"
        }
    }
    Ok(None) => "not authenticated (run 'systemprompt cloud login')",
    Err(_) => "not initialized",
};
```

---

## Profile Cloud Configuration

Profile YAML includes cloud settings:

```yaml
cloud:
  validation_mode: strict  # strict | warn | skip
  credentials_path: ~/.systemprompt/credentials.json
```

| Mode | Behavior |
|------|----------|
| strict | Fail at startup if credentials invalid |
| warn | Log warning, continue without credentials |
| skip | Skip credential validation entirely |

---

## See Also

- [profiles.md](../instructions/config/profiles.md) - Profile system documentation
- [credentials.md](../instructions/config/credentials.md) - Credentials documentation

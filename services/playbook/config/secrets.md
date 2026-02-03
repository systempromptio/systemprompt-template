---
title: "Secrets Management"
description: "Configure secrets loading from files or environment variables."
author: "SystemPrompt"
slug: "config-secrets"
keywords: "secrets, jwt, database, api-keys, environment, credentials"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Secrets Management

Configure secrets loading from files or environment variables.

> **Help**: `{ "command": "cloud secrets list" }` via `systemprompt_help`
> **Requires**: Profile configured -> See [Profiles Playbook](../profiles/index.md)

Secrets contain sensitive credentials: JWT signing keys, database URLs, and API keys.

---

## Secrets Struct

**Source**: `crates/shared/models/src/secrets.rs:12-35`

```rust
pub struct Secrets {
    pub jwt_secret: String,                    // Line 14 - Required
    pub database_url: String,                  // Line 16 - Required
    pub sync_token: Option<String>,            // Line 19 - Optional
    pub gemini: Option<String>,                // Line 22 - Gemini API key
    pub anthropic: Option<String>,             // Line 25 - Anthropic API key
    pub openai: Option<String>,                // Line 28 - OpenAI API key
    pub github: Option<String>,                // Line 31 - GitHub token
    #[serde(flatten)]
    pub custom: HashMap<String, String>,       // Line 34 - Additional secrets
}
```

### Field Details

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jwt_secret` | String | Yes | JWT signing key (min 32 chars) |
| `database_url` | String | Yes | PostgreSQL connection string |
| `sync_token` | Option | No | Cloud sync authentication |
| `gemini` | Option | No | Google Gemini API key |
| `anthropic` | Option | No | Anthropic Claude API key |
| `openai` | Option | No | OpenAI API key |
| `github` | Option | No | GitHub personal access token |
| `custom` | HashMap | No | Additional key-value secrets |

---

## SecretsBootstrap

**Source**: `crates/shared/models/src/secrets.rs:90-311`

### Static Storage

```rust
static SECRETS: OnceLock<Secrets> = OnceLock::new();        // Line 10
```

### Initialization

```rust
pub fn init() -> Result<&'static Secrets> {                 // Line 129
    let profile = ProfileBootstrap::get()?;                 // Requires ProfileBootstrap
    Self::load_from_profile_config(profile)
}
```

### Error Types

```rust
pub enum SecretsBootstrapError {
    NotInitialized,              // Secrets not loaded yet
    AlreadyInitialized,          // Already initialized
    ProfileNotInitialized,       // ProfileBootstrap must run first
    FileNotFound { path },       // Secrets file missing
    InvalidSecretsFile { message }, // Parse or validation error
    NoSecretsConfigured,         // No secrets section in profile
    JwtSecretRequired,           // JWT secret missing
    DatabaseUrlRequired,         // Database URL missing
}
```

---

## Loading Strategy

**Source**: `crates/shared/models/src/secrets.rs:203-248`

SecretsBootstrap determines how to load secrets based on:
1. Runtime environment (Fly.io container vs local)
2. Profile `secrets.source` setting

### Loading Flow

```
load_from_profile_config(profile)
    │
    ├─► IF subprocess OR FLY_APP_NAME set
    │       AND JWT_SECRET env var exists
    │   └─► load_from_env()
    │
    └─► ELSE check profile.secrets.source
            │
            ├─► SecretsSource::Env (in Fly.io)
            │   └─► load_from_env()
            │
            ├─► SecretsSource::Env (local)
            │   └─► TRY resolve_and_load_file()
            │       └─► FALLBACK load_from_env()
            │
            └─► SecretsSource::File
                └─► resolve_and_load_file()
```

---

## File-Based Loading

**Source**: `crates/shared/models/src/secrets.rs:286-305`

### File Format

```json
{
  "jwt_secret": "your-secret-key-minimum-32-characters-long",
  "database_url": "postgres://user:password@localhost:5432/dbname",
  "sync_token": "optional-sync-token",
  "gemini": "AIza...",
  "anthropic": "sk-ant-...",
  "openai": "sk-...",
  "github": "ghp_..."
}
```

### Path Resolution

The secrets path is resolved relative to the profile directory:

```rust
fn resolve_and_load_file(profile: &Profile, path_str: &str) -> Result<Secrets> {
    let profile_path = ProfileBootstrap::get_path()?;
    let profile_dir = Path::new(profile_path).parent().unwrap();
    let resolved = resolve_with_home(profile_dir, path_str);
    Self::load_from_file(&resolved)
}
```

**Resolution Rules**:
1. `~/path` - Expands to home directory
2. `./path` - Relative to profile directory
3. `/absolute/path` - Used as-is

### File Loading

```rust
fn load_from_file(path: &Path) -> Result<Secrets> {
    let content = fs::read_to_string(path)
        .map_err(|_| FileNotFound { path })?;

    let secrets: Secrets = serde_json::from_str(&content)
        .map_err(|e| InvalidSecretsFile { message: e.to_string() })?;

    secrets.validate()?;
    Ok(secrets)
}
```

---

## Environment-Based Loading

**Source**: `crates/shared/models/src/secrets.rs:155-201`

When `source: env` or running in a Fly.io container, secrets load from environment variables.

### Environment Variable Mapping

| Env Variable | Maps To |
|--------------|---------|
| `JWT_SECRET` | `secrets.jwt_secret` |
| `DATABASE_URL` | `secrets.database_url` |
| `SYNC_TOKEN` | `secrets.sync_token` |
| `GEMINI_API_KEY` | `secrets.gemini` |
| `ANTHROPIC_API_KEY` | `secrets.anthropic` |
| `OPENAI_API_KEY` | `secrets.openai` |
| `GITHUB_TOKEN` | `secrets.github` |

### Custom Secrets

Additional secrets can be loaded via `SYSTEMPROMPT_CUSTOM_SECRETS`:

```bash
export SYSTEMPROMPT_CUSTOM_SECRETS="MY_API_KEY,ANOTHER_SECRET"
export MY_API_KEY="value1"
export ANOTHER_SECRET="value2"
```

These are added to `secrets.custom` HashMap.

### Environment Loading Code

```rust
fn load_from_env() -> Result<Secrets> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| JwtSecretRequired)?;

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| DatabaseUrlRequired)?;

    let mut secrets = Secrets {
        jwt_secret,
        database_url,
        sync_token: std::env::var("SYNC_TOKEN").ok(),
        gemini: std::env::var("GEMINI_API_KEY").ok(),
        anthropic: std::env::var("ANTHROPIC_API_KEY").ok(),
        openai: std::env::var("OPENAI_API_KEY").ok(),
        github: std::env::var("GITHUB_TOKEN").ok(),
        custom: HashMap::new(),
    };

    // Load custom secrets
    if let Ok(custom_keys) = std::env::var("SYSTEMPROMPT_CUSTOM_SECRETS") {
        for key in custom_keys.split(',') {
            if let Ok(value) = std::env::var(key.trim()) {
                secrets.custom.insert(key.trim().to_string(), value);
            }
        }
    }

    secrets.validate()?;
    Ok(secrets)
}
```

---

## Validation

**Source**: `crates/shared/models/src/secrets.rs:39-56`

### JWT Secret Requirement

```rust
const JWT_SECRET_MIN_LENGTH: usize = 32;                    // Line 37

pub fn validate(&self) -> Result<()> {
    if self.jwt_secret.len() < JWT_SECRET_MIN_LENGTH {
        return Err(SecretsBootstrapError::InvalidSecretsFile {
            message: format!(
                "JWT secret must be at least {} characters",
                JWT_SECRET_MIN_LENGTH
            ),
        });
    }

    if self.database_url.is_empty() {
        return Err(SecretsBootstrapError::DatabaseUrlRequired);
    }

    Ok(())
}
```

### Validation Rules

| Field | Rule |
|-------|------|
| `jwt_secret` | Minimum 32 characters |
| `database_url` | Non-empty string |

---

## Validation Modes

The profile's `SecretsConfig` controls validation behavior:

**Source**: `crates/shared/models/src/profile/secrets.rs`

```rust
pub enum SecretsValidationMode {
    Strict,             // Fail on any validation error
    #[default] Warn,    // Log warning, continue
    Skip,               // Silent fallback
}
```

### Mode Behavior

| Mode | Missing File | Invalid Content | Validation Error |
|------|--------------|-----------------|------------------|
| `Strict` | Error | Error | Error |
| `Warn` | Warning + fallback | Warning + fallback | Warning + continue |
| `Skip` | Silent fallback | Silent fallback | Silent continue |

---

## Profile Configuration

Configure secrets in your profile's `secrets` section:

```yaml
# .systemprompt/profiles/local/profile.yaml
secrets:
  secrets_path: "./secrets.json"      # Relative to profile directory
  source: file                         # file | env
  validation: warn                     # strict | warn | skip
```

### Source Options

**`source: file`**
- Load from `secrets_path` file
- File must exist (unless validation mode allows fallback)

**`source: env`**
- Load from environment variables
- In local mode: tries file first, falls back to env
- In Fly.io: loads directly from env

---

## Secrets Access

### Get Methods

```rust
// Get entire secrets struct
SecretsBootstrap::get() -> Result<&'static Secrets>

// Convenience methods
SecretsBootstrap::jwt_secret() -> Result<&'static str>
SecretsBootstrap::database_url() -> Result<&'static str>
```

### Dynamic Secret Lookup

```rust
impl Secrets {
    pub fn get(&self, key: &str) -> Option<&String> {
        // Case-insensitive lookup for standard fields
        match key.to_lowercase().as_str() {
            "jwt_secret" => Some(&self.jwt_secret),
            "database_url" => Some(&self.database_url),
            "sync_token" => self.sync_token.as_ref(),
            "gemini" | "gemini_api_key" => self.gemini.as_ref(),
            "anthropic" | "anthropic_api_key" => self.anthropic.as_ref(),
            "openai" | "openai_api_key" => self.openai.as_ref(),
            "github" | "github_token" => self.github.as_ref(),
            _ => self.custom.get(key),
        }
    }
}
```

---

## Fly.io Container Detection

SecretsBootstrap automatically detects Fly.io containers:

```rust
fn is_fly_container() -> bool {
    std::env::var("FLY_APP_NAME").is_ok()
}
```

When running in a Fly.io container:
1. `FLY_APP_NAME` is set automatically
2. Secrets load from environment variables
3. File-based loading is skipped

---

## Secrets File Example

```json
{
  "jwt_secret": "your-super-secret-jwt-key-that-is-at-least-32-chars",
  "database_url": "postgres://systemprompt:password@localhost:5432/systemprompt",
  "sync_token": "sp_sync_abc123",
  "gemini": "AIzaSyA...",
  "anthropic": "sk-ant-api03-...",
  "openai": "sk-proj-...",
  "github": "ghp_abc123..."
}
```

### Generating a JWT Secret

```bash
# Generate 64-character random secret
openssl rand -base64 48

# Or using just
just generate-jwt-secret
```

---

## Troubleshooting

**"JWT secret must be at least 32 characters"**
- Generate a longer secret: `openssl rand -base64 48`
- Check for leading/trailing whitespace in JSON

**"Secrets file not found"**
- Verify path in profile's `secrets.secrets_path`
- Path is relative to profile directory
- Check file permissions

**"Database URL required"**
- Ensure `database_url` field exists in secrets.json
- For env mode, set `DATABASE_URL` environment variable

**"Profile not initialized"**
- SecretsBootstrap requires ProfileBootstrap to complete first
- Check `SYSTEMPROMPT_PROFILE` environment variable

**"Invalid secrets file"**
- Check JSON syntax (trailing commas, missing quotes)
- Validate with `jq . secrets.json`

---

## Security Best Practices

1. **File permissions**: Set secrets.json to `0600` (owner read/write only)
2. **Never commit**: Ensure `.gitignore` includes secrets files
3. **Separate per environment**: Different secrets for dev/staging/prod
4. **Rotate regularly**: Update API keys periodically
5. **Minimum length**: JWT secret should be 32+ characters

---

## Quick Reference

| Task | File Mode | Env Mode |
|------|-----------|----------|
| Set JWT secret | `secrets.json: jwt_secret` | `JWT_SECRET=...` |
| Set database URL | `secrets.json: database_url` | `DATABASE_URL=...` |
| Set Anthropic key | `secrets.json: anthropic` | `ANTHROPIC_API_KEY=...` |
| Set OpenAI key | `secrets.json: openai` | `OPENAI_API_KEY=...` |
| Set Gemini key | `secrets.json: gemini` | `GEMINI_API_KEY=...` |
| Set GitHub token | `secrets.json: github` | `GITHUB_TOKEN=...` |
| Add custom secret | `secrets.json: { "key": "value" }` | `SYSTEMPROMPT_CUSTOM_SECRETS=KEY` + `KEY=value` |
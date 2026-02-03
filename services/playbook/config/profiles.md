---
title: "Profile Configuration"
description: "Configure Profile struct, sub-configs, environment substitution, and validation."
author: "SystemPrompt"
slug: "config-profiles"
keywords: "profile, configuration, yaml, validation, paths, environment"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Profile Configuration

Configure Profile struct, sub-configs, environment substitution, and validation.

> **Help**: `{ "command": "cloud profile show" }` via `systemprompt_help`

Profiles are environment configurations stored in `.systemprompt/profiles/<name>/profile.yaml`.

---

## Profile Struct (Complete)

**Source**: `crates/shared/models/src/profile/mod.rs:88-120`

```rust
pub struct Profile {
    pub name: String,                          // Profile identifier
    pub display_name: String,                  // Human-readable name
    pub target: ProfileType,                   // Local | Cloud
    pub site: SiteConfig,                      // Site identity
    pub database: DatabaseConfig,              // Database settings
    pub server: ServerConfig,                  // HTTP server settings
    pub paths: PathsConfig,                    // Directory layout
    pub security: SecurityConfig,              // JWT configuration
    pub rate_limits: RateLimitsConfig,         // API rate limiting
    pub runtime: RuntimeConfig,                // Environment & logging
    pub cloud: Option<CloudConfig>,            // Cloud credentials paths
    pub secrets: Option<SecretsConfig>,        // Secrets configuration
    pub extensions: ExtensionsConfig,          // Extension management
}
```

### ProfileType Enum

```rust
pub enum ProfileType {
    #[default]
    Local,    // Development on local machine
    Cloud,    // Cloud deployment (Fly.io)
}
```

---

## Sub-Configuration Types

### SiteConfig

**Source**: `crates/shared/models/src/profile/site.rs`

```rust
pub struct SiteConfig {
    pub name: String,                          // Required: Site name
    pub github_link: Option<String>,           // Optional: GitHub URL
}
```

### DatabaseConfig

**Source**: `crates/shared/models/src/profile/database.rs:5-12`

```rust
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub db_type: String,                       // "postgres" or "postgresql"
    #[serde(default)]
    pub external_db_access: bool,              // Default: false
}
```

### ServerConfig

**Source**: `crates/shared/models/src/profile/server.rs:5-22`

```rust
pub struct ServerConfig {
    pub host: String,                          // Required: "127.0.0.1" or "0.0.0.0"
    pub port: u16,                             // Required: Must be > 0
    pub api_server_url: String,                // Required: e.g., "http://localhost:8080"
    pub api_internal_url: String,              // Required: Internal service URL
    pub api_external_url: String,              // Required: Public/external URL
    #[serde(default)]
    pub use_https: bool,                       // Default: false
    #[serde(default)]
    pub cors_allowed_origins: Vec<String>,     // Default: empty
}
```

### PathsConfig

**Source**: `crates/shared/models/src/profile/paths.rs:4-18`

```rust
pub struct PathsConfig {
    pub system: String,                        // Required: System root directory
    pub services: String,                      // Required: Services directory
    pub bin: String,                           // Required: Binaries directory
    pub web_path: Option<String>,              // Optional: Web output path
    pub storage: Option<String>,               // Optional: File storage
    pub geoip_database: Option<String>,        // Optional: MaxMind .mmdb path
}
```

**Derived Paths** (methods on PathsConfig):

| Method | Returns |
|--------|---------|
| `skills()` | `{services}/skills` |
| `config()` | `{services}/config/config.yaml` |
| `ai_config()` | `{services}/ai/config.yaml` |
| `content_config()` | `{services}/content/config.yaml` |
| `web_config()` | `{services}/web/config.yaml` |
| `web_metadata()` | `{services}/web/metadata.yaml` |
| `web_path_resolved()` | `web_path` or `{system}/web` |
| `storage_resolved()` | `storage` if set |
| `geoip_database_resolved()` | `geoip_database` if set |

### SecurityConfig

**Source**: `crates/shared/models/src/profile/security.rs:4-22`

```rust
pub struct SecurityConfig {
    #[serde(rename = "jwt_issuer")]
    pub issuer: String,                        // Required: JWT issuer claim
    #[serde(rename = "jwt_access_token_expiration")]
    pub access_token_expiration: i64,          // Required: Seconds (max 31,536,000)
    #[serde(rename = "jwt_refresh_token_expiration")]
    pub refresh_token_expiration: i64,         // Required: Seconds
    #[serde(rename = "jwt_audiences")]
    pub audiences: Vec<JwtAudience>,           // Required: Allowed audiences
}
```

**Token Expiration Limits**:
- `access_token_expiration`: Maximum 31,536,000 seconds (1 year)
- Default: 2,592,000 seconds (30 days)
- `refresh_token_expiration`: Default 15,552,000 seconds (180 days)

### RuntimeConfig

**Source**: `crates/shared/models/src/profile/runtime.rs:6-22`

```rust
pub struct RuntimeConfig {
    #[serde(default)]
    pub environment: Environment,              // Default: Development
    #[serde(default)]
    pub log_level: LogLevel,                   // Default: Normal
    #[serde(default)]
    pub output_format: OutputFormat,           // Default: Text
    #[serde(default)]
    pub no_color: bool,                        // Default: false
    #[serde(default)]
    pub non_interactive: bool,                 // Default: false
}
```

**Environment Enum**:
```rust
pub enum Environment {
    #[default] Development,
    Test,
    Staging,
    Production,
}
```

**LogLevel Enum** (with tracing filter mapping):
| Enum Value | Tracing Filter |
|------------|----------------|
| `Quiet` | `error` |
| `Normal` (default) | `info` |
| `Verbose` | `debug` |
| `Debug` | `trace` |

**OutputFormat Enum**:
```rust
pub enum OutputFormat {
    #[default] Text,
    Json,
    Yaml,
}
```

### RateLimitsConfig

**Source**: `crates/shared/models/src/profile/rate_limits.rs:58-100`

```rust
pub struct RateLimitsConfig {
    #[serde(default)]
    pub disabled: bool,                        // Default: false
    #[serde(default = "default_oauth_public")]
    pub oauth_public_per_second: u64,          // Default: 10
    #[serde(default = "default_oauth_auth")]
    pub oauth_auth_per_second: u64,            // Default: 10
    #[serde(default = "default_contexts")]
    pub contexts_per_second: u64,              // Default: 100
    #[serde(default = "default_tasks")]
    pub tasks_per_second: u64,                 // Default: 50
    #[serde(default = "default_artifacts")]
    pub artifacts_per_second: u64,             // Default: 50
    #[serde(default = "default_agent_registry")]
    pub agent_registry_per_second: u64,        // Default: 50
    #[serde(default = "default_agents")]
    pub agents_per_second: u64,                // Default: 20
    #[serde(default = "default_mcp_registry")]
    pub mcp_registry_per_second: u64,          // Default: 50
    #[serde(default = "default_mcp")]
    pub mcp_per_second: u64,                   // Default: 200
    #[serde(default = "default_stream")]
    pub stream_per_second: u64,                // Default: 100
    #[serde(default = "default_content")]
    pub content_per_second: u64,               // Default: 50
    #[serde(default = "default_burst")]
    pub burst_multiplier: u64,                 // Default: 3
    #[serde(default)]
    pub tier_multipliers: TierMultipliers,     // Default: TierMultipliers::default()
}
```

**TierMultipliers** (rate limit multipliers by user type):
```rust
pub struct TierMultipliers {
    #[serde(default = "default_admin_multiplier")]
    pub admin: f64,                            // Default: 10.0
    #[serde(default = "default_user_multiplier")]
    pub user: f64,                             // Default: 1.0
    #[serde(default = "default_a2a_multiplier")]
    pub a2a: f64,                              // Default: 5.0
    #[serde(default = "default_mcp_multiplier")]
    pub mcp: f64,                              // Default: 5.0
    #[serde(default = "default_service_multiplier")]
    pub service: f64,                          // Default: 5.0
    #[serde(default = "default_anon_multiplier")]
    pub anon: f64,                             // Default: 0.5
}
```

### CloudConfig

**Source**: `crates/shared/models/src/profile/cloud.rs:5-35`

```rust
pub struct CloudConfig {
    #[serde(default = "default_credentials_path")]
    pub credentials_path: String,              // Default: "./credentials.json"
    #[serde(default = "default_tenants_path")]
    pub tenants_path: String,                  // Default: "./tenants.json"
    pub tenant_id: Option<String>,             // Optional: Active tenant
    #[serde(default)]
    pub validation: CloudValidationMode,       // Default: Strict
}

pub enum CloudValidationMode {
    #[default] Strict,    // Fail on validation errors
    Warn,                 // Log warnings only
    Skip,                 // Skip validation entirely
}
```

### SecretsConfig

**Source**: `crates/shared/models/src/profile/secrets.rs:10-18`

```rust
pub struct SecretsConfig {
    pub secrets_path: String,                  // Path to secrets file
    #[serde(default)]
    pub validation: SecretsValidationMode,     // Default: Warn
    pub source: SecretsSource,                 // Required: File | Env
}

pub enum SecretsSource {
    File,    // Load from secrets_path file
    Env,     // Load from environment variables
}

pub enum SecretsValidationMode {
    Strict,             // Fail on missing secrets
    #[default] Warn,    // Log warnings only
    Skip,               // Skip validation entirely
}
```

### ExtensionsConfig

```rust
pub struct ExtensionsConfig {
    #[serde(default)]
    pub disabled: Vec<String>,                 // Extension IDs to disable
}
```

---

## Environment Variable Substitution

**Source**: `crates/shared/models/src/profile/mod.rs:52-68`

Profile YAML supports `${VAR_NAME}` syntax for environment variable substitution.

### Pattern

```yaml
database:
  type: postgres

server:
  host: ${HOST:-0.0.0.0}
  port: ${PORT}
  api_server_url: ${API_URL:-http://localhost:8080}
```

### Implementation

```rust
static ENV_VAR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{(\w+)\}").expect("Invalid regex")
});

fn substitute_env_vars(content: &str) -> String {
    ENV_VAR_REGEX.replace_all(content, |caps: &Captures| {
        let var_name = &caps[1];
        std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
    }).to_string()
}
```

### Rules

1. Pattern: `${VAR_NAME}` (alphanumeric and underscore only)
2. If env var not found, original `${VAR_NAME}` is retained
3. Substitution happens at parse time (before YAML parsing)
4. Default value syntax `${VAR:-default}` is handled by config manager, not profile parser

---

## Path Resolution

**Source**: `crates/shared/models/src/profile/paths.rs:69-108`

### Relative Path Resolution

Relative paths in profile are resolved relative to the profile directory:

```rust
pub fn resolve_path(base: &Path, path: &str) -> String {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_string_lossy().to_string()
    } else {
        base.join(path)
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| base.join(path).to_string_lossy().to_string())
    }
}
```

### Home Directory Expansion

Paths starting with `~/` are expanded:

```rust
pub fn expand_home(path_str: &str) -> PathBuf {
    if path_str.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE")) {
            return PathBuf::from(home).join(&path_str[2..]);
        }
    }
    PathBuf::from(path_str)
}
```

### Combined Resolution

```rust
pub fn resolve_with_home(base: &Path, path_str: &str) -> PathBuf {
    let expanded = expand_home(path_str);
    if expanded.is_absolute() {
        expanded
    } else {
        base.join(expanded)
    }
}
```

---

## Profile Validation

**Source**: `crates/shared/models/src/profile/validation.rs:13-32`

Validation runs after parsing and path resolution.

### Validation Chain

```rust
pub fn validate(&self) -> Result<()> {
    self.validate_required_fields()?;
    self.validate_paths()?;
    self.validate_security_settings()?;
    self.validate_cors_origins()?;
    self.validate_rate_limits()?;
    Ok(())
}
```

### 1. Required Fields Validation

**Lines 113-133**

| Field | Rule |
|-------|------|
| `name` | Non-empty string |
| `display_name` | Non-empty string |
| `site.name` | Non-empty string |
| `server.host` | Non-empty string |
| `server.port` | > 0 |
| `server.api_server_url` | Non-empty string |
| `server.api_internal_url` | Non-empty string |
| `server.api_external_url` | Non-empty string |

### 2. Paths Validation

**Lines 34-112**

**Local Profiles** (target = Local):
- `system`, `services`, `bin`: Must exist on filesystem
- `storage`, `geoip_database`, `web_path`: Must exist if specified

**Cloud Profiles** (target = Cloud):
- `system`, `services`, `bin`: Must be non-empty, start with `/app`
- `web_path`: Must start with `/app/web` (not `/app/services/web`)

### 3. Security Settings Validation

**Lines 141-149**

| Setting | Rule |
|---------|------|
| `access_token_expiration` | > 0 |
| `refresh_token_expiration` | > 0 |

### 4. CORS Origins Validation

**Lines 151-166**

- Each origin must be non-empty
- Each origin must start with `http://` or `https://`

### 5. Rate Limits Validation

**Lines 168-200**

If `disabled: false`:
- `burst_multiplier` > 0
- All rate limit values > 0

---

## Complete Profile Example

```yaml
# .systemprompt/profiles/local/profile.yaml
name: local
display_name: "Local Development"
target: local

site:
  name: "My Project"
  github_link: "https://github.com/org/repo"

database:
  type: postgres
  external_db_access: true

server:
  host: "0.0.0.0"
  port: 8080
  api_server_url: "http://localhost:8080"
  api_internal_url: "http://localhost:8080"
  api_external_url: "http://localhost:8080"
  use_https: false
  cors_allowed_origins:
    - "http://localhost:8080"
    - "http://localhost:5173"

paths:
  system: "/path/to/project"
  services: "/path/to/project/services"
  bin: "/path/to/project/target/release"
  web_path: "/path/to/project/web"
  storage: "/path/to/project/storage"

security:
  jwt_issuer: "systemprompt-local"
  jwt_access_token_expiration: 2592000
  jwt_refresh_token_expiration: 15552000
  jwt_audiences:
    - web
    - api
    - a2a
    - mcp

rate_limits:
  disabled: true

runtime:
  environment: development
  log_level: verbose
  output_format: text
  no_color: false
  non_interactive: false

cloud:
  credentials_path: "../../credentials.json"
  tenants_path: "../../tenants.json"
  tenant_id: local_19bff27604c

secrets:
  secrets_path: "./secrets.json"
  source: file
  validation: warn

extensions:
  disabled: []
```

---

## Troubleshooting

**"Required field missing"**
- Check all required fields are present in YAML
- Verify field names match exactly (case-sensitive)

**"Path does not exist"**
- For local profiles, all required paths must exist
- Check for typos in path strings
- Verify relative paths are relative to profile directory

**"Invalid CORS origin"**
- Origin must start with `http://` or `https://`
- Cannot be empty string

**"Rate limit validation failed"**
- All rate limits must be > 0 if not disabled
- Set `disabled: true` for development

---

## Quick Reference

| Config Type | Source File | Key Fields |
|-------------|-------------|------------|
| Profile | `mod.rs:88-120` | name, target, site, database, server, paths, security, rate_limits, runtime, cloud, secrets |
| SiteConfig | `site.rs` | name, github_link |
| DatabaseConfig | `database.rs` | db_type, external_db_access |
| ServerConfig | `server.rs` | host, port, api_server_url, api_internal_url, api_external_url, use_https, cors_allowed_origins |
| PathsConfig | `paths.rs` | system, services, bin, web_path, storage, geoip_database |
| SecurityConfig | `security.rs` | issuer, access_token_expiration, refresh_token_expiration, audiences |
| RuntimeConfig | `runtime.rs` | environment, log_level, output_format, no_color, non_interactive |
| RateLimitsConfig | `rate_limits.rs` | disabled, 12 rate fields, burst_multiplier, tier_multipliers |
| CloudConfig | `cloud.rs` | credentials_path, tenants_path, tenant_id, validation |
| SecretsConfig | `secrets.rs` | secrets_path, source, validation |
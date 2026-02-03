---
title: "Bootstrap Sequence"
description: "Initialize SystemPrompt through the 5-stage bootstrap sequence."
keywords:
  - bootstrap
  - initialization
  - profile
  - secrets
  - credentials
  - config
  - context
category: config
---

# Bootstrap Sequence

Initialize SystemPrompt through the 5-stage bootstrap sequence.

> **Help**: `{ "command": "admin config show" }` via `systemprompt_help`

---

## The 5-Stage Bootstrap Sequence

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         BOOTSTRAP SEQUENCE                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Stage 1: ProfileBootstrap                                               │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  SYSTEMPROMPT_PROFILE env var                                     │   │
│  │         ↓                                                         │   │
│  │  Load YAML from disk                                              │   │
│  │         ↓                                                         │   │
│  │  Substitute ${ENV_VAR} patterns                                   │   │
│  │         ↓                                                         │   │
│  │  Resolve relative paths                                           │   │
│  │         ↓                                                         │   │
│  │  Validate profile (paths, security, rate limits)                  │   │
│  │         ↓                                                         │   │
│  │  Store in OnceLock<Profile>                                       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              ↓                                           │
│  Stage 2: SecretsBootstrap                                               │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Check SecretsSource (File | Env)                                 │   │
│  │         ↓                                                         │   │
│  │  IF Fly.io container → Load from environment                      │   │
│  │  ELSE → Load from secrets.json file                               │   │
│  │         ↓                                                         │   │
│  │  Validate JWT secret (min 32 chars)                               │   │
│  │         ↓                                                         │   │
│  │  Validate DATABASE_URL exists                                     │   │
│  │         ↓                                                         │   │
│  │  Store in OnceLock<Secrets>                                       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              ↓                                           │
│  Stage 3: CredentialsBootstrap (Optional)                                │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Load credentials.json from profile.cloud.credentials_path        │   │
│  │         ↓                                                         │   │
│  │  Validate token not expired (24 hour limit)                       │   │
│  │         ↓                                                         │   │
│  │  Validate with Cloud API                                          │   │
│  │         ↓                                                         │   │
│  │  Store in OnceLock<Option<CloudCredentials>>                      │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              ↓                                           │
│  Stage 4: Config                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Aggregate Profile + Secrets                                      │   │
│  │         ↓                                                         │   │
│  │  Canonicalize all paths                                           │   │
│  │         ↓                                                         │   │
│  │  Validate YAML files exist (config.yaml, content_config.yaml)     │   │
│  │         ↓                                                         │   │
│  │  Validate database type = postgres                                │   │
│  │         ↓                                                         │   │
│  │  Store in OnceLock<Config>                                        │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              ↓                                           │
│  Stage 5: AppContext                                                     │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Initialize database connection pool                              │   │
│  │         ↓                                                         │   │
│  │  Discover extensions via inventory crate                          │   │
│  │         ↓                                                         │   │
│  │  Load optional GeoIP database                                     │   │
│  │         ↓                                                         │   │
│  │  Load optional content config                                     │   │
│  │         ↓                                                         │   │
│  │  Initialize analytics, fingerprinting, user services              │   │
│  │         ↓                                                         │   │
│  │  Return Arc<AppContext>                                           │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Stage 1: ProfileBootstrap

**Source**: `crates/shared/models/src/profile_bootstrap.rs`

ProfileBootstrap loads and validates the profile YAML file. This is the foundation for all configuration.

### Static Storage

```rust
static PROFILE: OnceLock<Profile> = OnceLock::new();       // Line 7
static PROFILE_PATH: OnceLock<String> = OnceLock::new();   // Line 8
```

### Initialization Flow

```rust
pub fn init() -> Result<&'static Profile> {                 // Line 32
    let profile_path = std::env::var("SYSTEMPROMPT_PROFILE")
        .map_err(|_| ProfileBootstrapError::PathNotSet)?;

    Self::load_from_path_and_validate(&profile_path)
}
```

### Key Functions

| Function | Line | Purpose |
|----------|------|---------|
| `init()` | 32-55 | Main initialization from SYSTEMPROMPT_PROFILE |
| `init_from_path(path)` | 84-103 | Initialize from explicit path |
| `try_init()` | 77-82 | Idempotent initialization (returns existing if already set) |
| `get()` | 57-64 | Get initialized profile |
| `get_path()` | 66-73 | Get profile file path |
| `is_initialized()` | 75 | Check if profile is loaded |

### Profile Loading Sequence

1. Read `SYSTEMPROMPT_PROFILE` environment variable
2. Load YAML content from file
3. Call `substitute_env_vars(content)` - replaces `${VAR_NAME}` patterns
4. Parse YAML with `serde_yaml::from_str()`
5. Call `paths.resolve_relative_to(profile_dir)` - resolve relative paths
6. Call `profile.validate()` - comprehensive validation
7. Store in `OnceLock<Profile>`

### Error Types

```rust
pub enum ProfileBootstrapError {
    NotInitialized,                    // Profile not yet loaded
    AlreadyInitialized,                // Already initialized
    PathNotSet,                        // SYSTEMPROMPT_PROFILE not set
    ValidationFailed(String),          // Validation errors
    LoadFailed(String),                // File read/parse errors
}
```

---

## Stage 2: SecretsBootstrap

**Source**: `crates/shared/models/src/secrets.rs`

SecretsBootstrap loads sensitive credentials. It supports loading from files or environment variables.

### Static Storage

```rust
static SECRETS: OnceLock<Secrets> = OnceLock::new();        // Line 10
```

### Secrets Struct

```rust
pub struct Secrets {
    pub jwt_secret: String,                    // Required, min 32 chars
    pub database_url: String,                  // Required
    pub sync_token: Option<String>,
    pub gemini: Option<String>,                // Gemini API key
    pub anthropic: Option<String>,             // Anthropic API key
    pub openai: Option<String>,                // OpenAI API key
    pub github: Option<String>,                // GitHub token
    pub custom: HashMap<String, String>,       // Additional secrets
}
```

### Initialization Flow

```rust
pub fn init() -> Result<&'static Secrets> {                 // Line 129
    let profile = ProfileBootstrap::get()?;                 // Requires Stage 1
    Self::load_from_profile_config(profile)
}
```

### Loading Priority

1. **Check environment**: If `FLY_APP_NAME` is set or subprocess mode, load from env
2. **Check source**: Profile specifies `SecretsSource::File` or `SecretsSource::Env`
3. **File loading**: Resolve path relative to profile directory, read JSON
4. **Env loading**: Read from `JWT_SECRET`, `DATABASE_URL`, `GEMINI_API_KEY`, etc.

### Environment Variables (when source = Env)

| Env Variable | Maps To |
|--------------|---------|
| `JWT_SECRET` | `secrets.jwt_secret` |
| `DATABASE_URL` | `secrets.database_url` |
| `SYNC_TOKEN` | `secrets.sync_token` |
| `GEMINI_API_KEY` | `secrets.gemini` |
| `ANTHROPIC_API_KEY` | `secrets.anthropic` |
| `OPENAI_API_KEY` | `secrets.openai` |
| `GITHUB_TOKEN` | `secrets.github` |

### Validation Rules

- `jwt_secret`: Minimum 32 characters (`JWT_SECRET_MIN_LENGTH = 32`)
- `database_url`: Required, non-empty

### Validation Modes

Profile specifies validation behavior via `SecretsValidationMode`:

| Mode | Behavior |
|------|----------|
| `Strict` | Fail on any validation error |
| `Warn` | Log warning, continue |
| `Skip` | Silent fallback |

---

## Stage 3: CredentialsBootstrap (Optional)

**Source**: `crates/infra/cloud/src/credentials_bootstrap.rs`

CredentialsBootstrap loads cloud API credentials. This stage is optional - local-only deployments skip it.

### Static Storage

```rust
static CREDENTIALS: OnceLock<Option<CloudCredentials>> = OnceLock::new();
```

### CloudCredentials Struct

```rust
pub struct CloudCredentials {
    pub api_token: String,                     // JWT token
    pub api_url: String,                       // Cloud API endpoint
    pub authenticated_at: DateTime<Utc>,       // Login timestamp
    pub user_email: String,                    // User email
}
```

### Initialization Flow (Async)

```rust
pub async fn init() -> Result<Option<&'static CloudCredentials>> {
    if std::env::var("FLY_APP_NAME").is_ok() {
        // Container mode: load from environment
        Self::load_from_env()
    } else {
        // Local mode: load from credentials.json
        Self::load_credentials_from_path()
    }
}
```

### Token Expiration

- Tokens expire 24 hours after `authenticated_at`
- Warning issued when token expires within 1 hour
- Error if token already expired

### API Validation

After loading, credentials are validated against the Cloud API:

```rust
let client = CloudApiClient::new(&credentials.api_url, &credentials.api_token);
client.get_user().await?;  // GET /api/v1/auth/me
```

---

## Stage 4: Config

**Source**: `crates/shared/models/src/config/mod.rs`

Config aggregates Profile and Secrets into a single runtime configuration object.

### Static Storage

```rust
static CONFIG: OnceLock<Config> = OnceLock::new();
```

### Config Struct

```rust
pub struct Config {
    // From Profile.site
    pub sitename: String,
    pub github_link: String,

    // From Secrets
    pub database_type: String,
    pub database_url: String,
    pub github_token: Option<String>,

    // From Profile.paths (canonicalized)
    pub system_path: String,
    pub services_path: String,
    pub bin_path: String,
    pub skills_path: String,
    pub settings_path: String,
    pub content_config_path: String,
    pub geoip_database_path: Option<String>,
    pub web_path: String,
    pub web_config_path: String,
    pub web_metadata_path: String,

    // From Profile.server
    pub host: String,
    pub port: u16,
    pub api_server_url: String,
    pub api_internal_url: String,
    pub api_external_url: String,
    pub use_https: bool,
    pub cors_allowed_origins: Vec<String>,

    // From Profile.security
    pub jwt_issuer: String,
    pub jwt_access_token_expiration: i64,
    pub jwt_refresh_token_expiration: i64,
    pub jwt_audiences: Vec<JwtAudience>,

    // From Profile.rate_limits
    pub rate_limits: RateLimitConfig,

    // Profile type
    pub is_cloud: bool,
}
```

### Initialization Flow

```rust
pub fn init() -> Result<&'static Config> {                  // Line 75
    let profile = ProfileBootstrap::get()?;                 // Requires Stage 1
    Self::from_profile(profile)                             // Requires Stage 2
}
```

### YAML File Validation

Config validates that required YAML files exist:

- `{services}/config/config.yaml`
- `{services}/content/config.yaml`
- `{services}/web/config.yaml`
- `{services}/web/metadata.yaml`

### Database Validation

Only PostgreSQL is supported:

```rust
pub fn validate_database_config(&self) -> Result<()> {
    match self.database_type.as_str() {
        "postgres" | "postgresql" => Ok(()),
        other => Err(ConfigError::UnsupportedDatabaseType(other.to_string()))
    }
}
```

---

## Stage 5: AppContext

**Source**: `crates/app/runtime/src/context.rs`

AppContext initializes all runtime services including database, extensions, and analytics.

### AppContext Struct

```rust
pub struct AppContext {
    config: Arc<Config>,
    database: DbPool,
    api_registry: Arc<ModuleApiRegistry>,
    extension_registry: Arc<ExtensionRegistry>,
    geoip_reader: Option<GeoIpReader>,
    content_config: Option<Arc<ContentConfigRaw>>,
    route_classifier: Arc<RouteClassifier>,
    analytics_service: Arc<AnalyticsService>,
    fingerprint_repo: Option<Arc<FingerprintRepository>>,
    user_service: Option<Arc<UserService>>,
}
```

### Initialization Flow (Async)

```rust
pub async fn new() -> Result<Self> {                        // Line 49
    Self::builder().build().await
}

async fn new_internal(...) -> Result<Self> {                // Line 58
    let profile = ProfileBootstrap::get()?;
    AppPaths::init(&profile.paths)?;
    FilesConfig::init()?;

    let config = Arc::new(Config::get()?.clone());
    let database = Database::from_config(&config.database_type, &config.database_url).await?;
    let extension_registry = ExtensionRegistry::discover()?;

    // Optional components (warn on failure, continue)
    let geoip_reader = load_geoip_database(&config).ok();
    let content_config = load_content_config(&profile).ok();

    // Services
    let analytics_service = AnalyticsService::new(&database);
    let fingerprint_repo = FingerprintRepository::new(&database).ok();
    let user_service = UserService::new(&database).ok();

    init_logging(&profile.runtime)?;

    Ok(Self { ... })
}
```

### Builder Pattern

```rust
pub fn builder() -> AppContextBuilder {
    AppContextBuilder::new()
}

impl AppContextBuilder {
    pub fn with_extensions(mut self, registry: ExtensionRegistry) -> Self;
    pub fn with_startup_warnings(mut self, show: bool) -> Self;
    pub async fn build(self) -> Result<AppContext>;
}
```

---

## OnceLock Singleton Pattern

All bootstrap components use `std::sync::OnceLock` for thread-safe, initialize-once semantics.

### Pattern Structure

```rust
static COMPONENT: OnceLock<T> = OnceLock::new();

impl ComponentBootstrap {
    pub fn init() -> Result<&'static T> {
        COMPONENT.get_or_try_init(|| {
            // Initialization logic
            Ok(initialized_value)
        })
    }

    pub fn get() -> Result<&'static T> {
        COMPONENT.get().ok_or(NotInitialized)
    }

    pub fn try_init() -> Result<&'static T> {
        if Self::is_initialized() {
            Self::get()
        } else {
            Self::init()
        }
    }

    pub fn is_initialized() -> bool {
        COMPONENT.get().is_some()
    }
}
```

### Benefits

- **Thread-safe**: Multiple threads can safely call `init()`
- **Initialize-once**: Value is computed exactly once
- **Panic-free**: Returns `Result` instead of panicking
- **No re-initialization**: `AlreadyInitialized` error if called twice

---

## Dependency Chain

```
ProfileBootstrap (required)
       ↓
SecretsBootstrap (required)
       ↓
CredentialsBootstrap (optional)
       ↓
Config (required)
       ↓
AppContext (required)
```

Each stage explicitly checks its prerequisites:

```rust
// SecretsBootstrap checks ProfileBootstrap
let profile = ProfileBootstrap::get()
    .map_err(|_| SecretsBootstrapError::ProfileNotInitialized)?;

// Config checks both
let profile = ProfileBootstrap::get()?;
let secrets = SecretsBootstrap::get()?;
```

---

## Error Handling Summary

| Stage | Failure Behavior |
|-------|-----------------|
| ProfileBootstrap | Fatal - application cannot start |
| SecretsBootstrap | Configurable via validation mode |
| CredentialsBootstrap | Warn - cloud features disabled |
| Config | Fatal - application cannot start |
| AppContext | Optional components warn, core is fatal |

---

## Troubleshooting

**"Profile not initialized"**
- Check `SYSTEMPROMPT_PROFILE` environment variable
- Verify profile file exists at specified path

**"Secrets not initialized"**
- Ensure ProfileBootstrap completed first
- Check `secrets.json` path in profile
- Verify JWT secret is at least 32 characters

**"Credentials expired"**
- Re-authenticate with `just login`
- Token expires 24 hours after login

**"Config validation failed"**
- Check all required YAML files exist
- Verify database type is "postgres"
- Check path permissions

**"AppContext failed"**
- Verify database is running and accessible
- Check DATABASE_URL format
- Review extension registration

---

## Quick Reference

| Stage | Source File | Static | Async |
|-------|-------------|--------|-------|
| 1. ProfileBootstrap | `profile_bootstrap.rs` | `PROFILE`, `PROFILE_PATH` | No |
| 2. SecretsBootstrap | `secrets.rs` | `SECRETS` | No |
| 3. CredentialsBootstrap | `credentials_bootstrap.rs` | `CREDENTIALS` | Yes |
| 4. Config | `config/mod.rs` | `CONFIG` | No |
| 5. AppContext | `context.rs` | None (Arc) | Yes |

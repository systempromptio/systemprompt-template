# SystemPrompt Profile System

Profiles are the SINGLE source of truth for all configuration. Environment variables should only be used to specify which profile to load.

---

## Profile Files

Location: `services/profiles/{name}.secrets.profile.yml`

```yaml
name: local
display_name: Local Development
database:
  url: postgres://user:pass@localhost:5432/db
server:
  host: 127.0.0.1
  port: 8080
  api_server_url: http://localhost:8080
paths:
  services: /path/to/services
security:
  jwt_secret: ...
api_keys:
  gemini: ...
cloud:
  validation_mode: strict
  credentials_path: ~/.systemprompt/credentials.json
```

---

## Initialization Flow

```
SYSTEMPROMPT_PROFILE=/path/to/profile.yml
         │
         ▼
ProfileBootstrap::init()
         │ (loads and validates profile)
         ▼
CredentialsBootstrap::init()
         │ (loads cloud credentials)
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
Application Ready
```

---

## Correct MCP Server Bootstrap

```rust
use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt::credentials::CredentialsBootstrap;
use systemprompt::profile::ProfileBootstrap;
use systemprompt::models::Config;
use systemprompt::system::AppContext;
use systemprompt::identifiers::McpServerId;
use systemprompt::logging;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    ProfileBootstrap::init().context("Profile initialization failed")?;
    CredentialsBootstrap::init().context("Cloud credentials initialization failed")?;
    Config::init().context("Failed to initialize configuration")?;

    let ctx = Arc::new(AppContext::new().await?);
    logging::init_logging(ctx.db_pool().clone());

    let config = Config::global();
    let port = config.port;
    let service_id = McpServerId::new("systemprompt-infrastructure");

    let server = InfrastructureServer::new(ctx.db_pool().clone(), service_id.clone(), ctx.clone());
    let router = systemprompt::mcp::create_router(server, ctx.clone()).await?;

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(service_id = %service_id, addr = %addr, "MCP server listening");
    axum::serve(listener, router).await?;

    Ok(())
}
```

---

## Forbidden Patterns

| Pattern | Why Forbidden |
|---------|---------------|
| `dotenvy::dotenv()` | Use profiles, not .env files |
| `env::var("CONFIG_VALUE")` | Config comes from profile |
| `unwrap_or_else(\|\| default)` | Fail explicitly if config missing |
| `unwrap_or_default()` | No fuzzy defaults |
| Hardcoded fallback values | Profile must provide all values |

---

## Accessing Configuration

```rust
let config = Config::global();

let db_url = &config.database.url;
let api_url = &config.server.api_server_url;
let jwt_secret = &config.security.jwt_secret;

let mcp = config.mcp_server("my-server")?;
let port = mcp.port;
```

---

## Environment Variables

Only ONE env var is used at runtime:

| Variable | Purpose |
|----------|---------|
| `SYSTEMPROMPT_PROFILE` | Path to profile YAML file |

All other configuration comes from the profile file.

---

## See Also

- [credentials.md](./credentials.md) - Cloud credentials documentation
- [services.md](./services.md) - MCP server configuration in profiles
- [../architecture/overview.md](../architecture/overview.md) - Extension architecture

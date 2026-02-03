---
title: "Server Configuration"
description: "Configure HTTP server settings including host, port, CORS, and HTTPS."
author: "SystemPrompt"
slug: "config-server"
keywords: "server, host, port, cors, https, api, url"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Server Configuration

Configure HTTP server settings including host, port, CORS, and HTTPS.

> **Help**: `{ "command": "admin config server show" }` via `systemprompt_help`
> **Requires**: Profile configured -> See [Profiles Playbook](../profiles/index.md)

ServerConfig defines HTTP server settings: host, port, API URLs, CORS, and HTTPS.

---

## ServerConfig Struct

**Source**: `crates/shared/models/src/profile/server.rs:5-22`

```rust
pub struct ServerConfig {
    pub host: String,                          // Required: Bind address
    pub port: u16,                             // Required: Listen port
    pub api_server_url: String,                // Required: Primary API URL
    pub api_internal_url: String,              // Required: Internal service URL
    pub api_external_url: String,              // Required: Public/external URL
    #[serde(default)]
    pub use_https: bool,                       // Default: false
    #[serde(default)]
    pub cors_allowed_origins: Vec<String>,     // Default: empty
}
```

### Field Details

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `host` | String | Yes | - | Network interface to bind |
| `port` | u16 | Yes | - | TCP port number (> 0) |
| `api_server_url` | String | Yes | - | Primary API endpoint |
| `api_internal_url` | String | Yes | - | Internal service-to-service URL |
| `api_external_url` | String | Yes | - | Public URL for external clients |
| `use_https` | bool | No | `false` | Enable HTTPS |
| `cors_allowed_origins` | Vec | No | `[]` | Allowed CORS origins |

---

## Host Configuration

### Common Values

| Value | Use Case |
|-------|----------|
| `127.0.0.1` | Local development only |
| `0.0.0.0` | Accept connections from any interface |
| Specific IP | Bind to specific network interface |

### Example

```yaml
server:
  host: "0.0.0.0"  # Accept external connections
  port: 8080
```

---

## The 3 API URLs

SystemPrompt uses three distinct URLs for different purposes.

### api_server_url

**Primary API endpoint** used for:
- Client applications
- Browser requests
- Default URL when others not specified

```yaml
server:
  api_server_url: "http://localhost:8080"
```

### api_internal_url

**Internal service-to-service communication**:
- Background jobs calling API
- Health checks
- Internal service mesh

```yaml
server:
  api_internal_url: "http://localhost:8080"
```

In cloud deployments, this might be a private network address:

```yaml
server:
  api_internal_url: "http://sp-tenant.internal:8080"
```

### api_external_url

**Public URL for external access**:
- OAuth callbacks
- Webhook URLs
- Public documentation links
- External API references

```yaml
server:
  api_external_url: "https://api.example.com"
```

### URL Relationship

| Scenario | api_server_url | api_internal_url | api_external_url |
|----------|----------------|------------------|------------------|
| Local dev | `http://localhost:8080` | `http://localhost:8080` | `http://localhost:8080` |
| Docker | `http://0.0.0.0:8080` | `http://app:8080` | `http://localhost:8080` |
| Production | `https://api.example.com` | `http://internal:8080` | `https://api.example.com` |

---

## CORS Configuration

Cross-Origin Resource Sharing (CORS) allows frontend applications to make API requests.

### Basic Setup

```yaml
server:
  cors_allowed_origins:
    - "http://localhost:8080"       # API origin
    - "http://localhost:5173"       # Vite dev server
    - "http://localhost:3000"       # React dev server
```

### Production Setup

```yaml
server:
  cors_allowed_origins:
    - "https://app.example.com"
    - "https://www.example.com"
```

### Validation Rules

**Source**: `crates/shared/models/src/profile/validation.rs:151-166`

Each CORS origin must:
1. Be non-empty
2. Start with `http://` or `https://`

```rust
fn validate_cors_origins(&self) -> Result<()> {
    for origin in &self.server.cors_allowed_origins {
        if origin.is_empty() {
            return Err(ProfileError::EmptyCorsOrigin);
        }
        if !origin.starts_with("http://") && !origin.starts_with("https://") {
            return Err(ProfileError::InvalidCorsOrigin(origin.clone()));
        }
    }
    Ok(())
}
```

### Common Mistakes

| Invalid | Why | Fix |
|---------|-----|-----|
| `localhost:5173` | Missing protocol | `http://localhost:5173` |
| `http://localhost:5173/` | Trailing slash | `http://localhost:5173` |
| `*` | Wildcard not supported | List specific origins |
| Empty string | Not allowed | Remove or specify origin |

---

## HTTPS Configuration

### Enable HTTPS

```yaml
server:
  use_https: true
  api_server_url: "https://api.example.com"
  api_external_url: "https://api.example.com"
```

### Development (No HTTPS)

```yaml
server:
  use_https: false
  api_server_url: "http://localhost:8080"
```

### Cloud Deployment

In cloud deployments, TLS is typically terminated at the proxy:

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  use_https: false                    # TLS at proxy level
  api_server_url: "http://0.0.0.0:8080"
  api_internal_url: "http://0.0.0.0:8080"
  api_external_url: "https://tenant.systemprompt.io"
```

---

## Complete Configuration Examples

### Local Development

```yaml
server:
  host: "127.0.0.1"
  port: 8080
  api_server_url: "http://localhost:8080"
  api_internal_url: "http://localhost:8080"
  api_external_url: "http://localhost:8080"
  use_https: false
  cors_allowed_origins:
    - "http://localhost:8080"
    - "http://localhost:5173"
```

### Docker Development

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  api_server_url: "http://0.0.0.0:8080"
  api_internal_url: "http://systemprompt:8080"
  api_external_url: "http://localhost:8080"
  use_https: false
  cors_allowed_origins:
    - "http://localhost:8080"
    - "http://localhost:5173"
```

### Production Cloud

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  api_server_url: "https://api.example.com"
  api_internal_url: "http://app.internal:8080"
  api_external_url: "https://api.example.com"
  use_https: false                    # TLS at load balancer
  cors_allowed_origins:
    - "https://app.example.com"
    - "https://www.example.com"
```

---

## Validation Rules

**Source**: `crates/shared/models/src/profile/validation.rs:113-133`

### Required Fields

All server fields are required:

```rust
fn validate_required_fields(&self) -> Result<()> {
    if self.server.host.is_empty() {
        return Err(ProfileError::MissingField("server.host"));
    }
    if self.server.port == 0 {
        return Err(ProfileError::InvalidPort);
    }
    if self.server.api_server_url.is_empty() {
        return Err(ProfileError::MissingField("server.api_server_url"));
    }
    if self.server.api_internal_url.is_empty() {
        return Err(ProfileError::MissingField("server.api_internal_url"));
    }
    if self.server.api_external_url.is_empty() {
        return Err(ProfileError::MissingField("server.api_external_url"));
    }
    Ok(())
}
```

### Port Validation

Port must be greater than 0:

```rust
if self.server.port == 0 {
    return Err(ProfileError::InvalidPort);
}
```

---

## Environment Variables

When using `Profile::from_env()`:

| Env Variable | Maps To |
|--------------|---------|
| `HOST` | `server.host` |
| `PORT` | `server.port` |
| `API_SERVER_URL` | `server.api_server_url` |
| `API_INTERNAL_URL` | `server.api_internal_url` |
| `API_EXTERNAL_URL` | `server.api_external_url` |
| `USE_HTTPS` | `server.use_https` |
| `CORS_ALLOWED_ORIGINS` | `server.cors_allowed_origins` (comma-separated) |

---

## Config Access

After bootstrap, server config is available via Config struct:

```rust
let config = Config::get()?;
println!("Host: {}", config.host);
println!("Port: {}", config.port);
println!("API URL: {}", config.api_server_url);
println!("Internal URL: {}", config.api_internal_url);
println!("External URL: {}", config.api_external_url);
println!("HTTPS: {}", config.use_https);
println!("CORS: {:?}", config.cors_allowed_origins);
```

---

## Troubleshooting

**"Port already in use"**
- Another process is using the port
- Check with `lsof -i :8080`
- Change port or stop conflicting process

**"CORS error in browser"**
- Add frontend origin to `cors_allowed_origins`
- Check for typos (no trailing slashes)
- Ensure protocol matches (http vs https)

**"Connection refused"**
- Check `host` is `0.0.0.0` for external access
- Verify port is not firewalled
- Check service is running

**"Invalid CORS origin"**
- Origin must start with `http://` or `https://`
- Cannot be empty string
- No wildcards (`*`)

**"api_internal_url missing"**
- All three URL fields are required
- Even if same as api_server_url

---

## Quick Reference

| Setting | Development | Production |
|---------|-------------|------------|
| `host` | `127.0.0.1` | `0.0.0.0` |
| `port` | 8080 | 8080 |
| `api_server_url` | `http://localhost:8080` | `https://api.example.com` |
| `api_internal_url` | `http://localhost:8080` | `http://internal:8080` |
| `api_external_url` | `http://localhost:8080` | `https://api.example.com` |
| `use_https` | `false` | `false` (TLS at LB) |
| `cors_allowed_origins` | Dev URLs | Production domains |
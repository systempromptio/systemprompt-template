# systemprompt-infrastructure

[![Crates.io](https://img.shields.io/crates/v/systemprompt-infrastructure.svg)](https://crates.io/crates/systemprompt-infrastructure)
[![Documentation](https://docs.rs/systemprompt-infrastructure/badge.svg)](https://docs.rs/systemprompt-infrastructure)
[![CI](https://github.com/systempromptio/systemprompt-infrastructure/actions/workflows/ci.yaml/badge.svg)](https://github.com/systempromptio/systemprompt-infrastructure/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/systempromptio/systemprompt-infrastructure/branch/main/graph/badge.svg)](https://codecov.io/gh/systempromptio/systemprompt-infrastructure)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MCP server for managing SystemPrompt cloud infrastructure — sync, deploy, and monitor your deployments.

## Part of the SystemPrompt Ecosystem

This crate is part of the [SystemPrompt](https://systemprompt.io) platform. For comprehensive documentation, tutorials, and support, visit **[systemprompt.io](https://systemprompt.io)**.

- [SystemPrompt](https://systemprompt.io) - Main project website
- [Documentation](https://systemprompt.io/docs) - Full documentation
- [Getting Started](https://systemprompt.io/docs/getting-started) - Quick start guide

## Features

- **Cloud Sync** - Bidirectional sync between local and cloud (files, database, content, skills)
- **One-Command Deploy** - Build Rust binary, Docker image, and deploy to Fly.io
- **Export/Backup** - Export content and skills to disk for backup
- **Status Monitoring** - Check cloud connectivity and deployment status
- **Configuration Display** - View and filter current configuration

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
systemprompt-infrastructure = "0.1"
```

Or install via cargo:

```bash
cargo add systemprompt-infrastructure
```

## Quick Start

```rust
use systemprompt_infrastructure::InfrastructureServer;
use systemprompt::system::AppContext;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ctx = Arc::new(AppContext::new().await?);

    let server = InfrastructureServer::new(
        ctx.db_pool().clone(),
        "systemprompt-infrastructure".into(),
        ctx.clone(),
    );

    // Server is ready to handle MCP requests
    Ok(())
}
```

For complete setup instructions, see the [SystemPrompt documentation](https://systemprompt.io/docs).

## MCP Configuration

### Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "systemprompt-infrastructure": {
      "command": "systemprompt-infrastructure",
      "env": {
        "DATABASE_URL": "postgresql://localhost/systemprompt",
        "SYSTEMPROMPT_TENANT_ID": "your-tenant-id",
        "SYSTEMPROMPT_API_TOKEN": "your-api-token"
      }
    }
  }
}
```

### HTTP Transport

The server supports HTTP transport with SSE:

```bash
MCP_PORT=5010 systemprompt-infrastructure
# Listens on http://0.0.0.0:5010
```

## Available Tools

| Tool | Description |
|------|-------------|
| `sync` | Sync files, database, content, or skills between local and cloud |
| `export` | Export content and skills to disk for backup |
| `deploy` | Build and deploy to Fly.io (cargo build → Docker → deploy) |
| `status` | Get cloud connection state and deployment status |
| `config` | Display current configuration by section |

### Tool Examples

#### Sync

```json
// Push all local changes to cloud
{ "name": "sync", "arguments": { "target": "all", "direction": "push" } }

// Preview database sync (dry run)
{ "name": "sync", "arguments": { "target": "database", "direction": "push", "dry_run": true } }

// Pull cloud state to local
{ "name": "sync", "arguments": { "target": "files", "direction": "pull" } }
```

#### Deploy

```json
// Full build and deploy
{ "name": "deploy", "arguments": {} }

// Skip build, redeploy existing
{ "name": "deploy", "arguments": { "skip_build": true } }
```

#### Export

```json
// Export all content and skills to disk
{ "name": "export", "arguments": { "target": "all" } }
```

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `SYSTEMPROMPT_TENANT_ID` | Yes | - | Your tenant identifier |
| `SYSTEMPROMPT_API_TOKEN` | Yes | - | API authentication token |
| `SYSTEMPROMPT_API_URL` | No | `https://api.systemprompt.io` | API endpoint |
| `MCP_PORT` | No | `5010` | HTTP server port |
| `MCP_SERVICE_ID` | No | `systemprompt-infrastructure` | Service identifier |

Get your API credentials at [systemprompt.io](https://systemprompt.io).

## Architecture

```
┌─────────────────┐     ┌──────────────────────────┐
│  MCP Client     │────▶│  Infrastructure Server   │
│  (Claude, etc)  │     │  (HTTP + SSE)            │
└─────────────────┘     └────────────┬─────────────┘
                                     │
                    ┌────────────────┼────────────────┐
                    ▼                ▼                ▼
              ┌──────────┐    ┌──────────┐    ┌──────────┐
              │   Sync   │    │  Deploy  │    │  Export  │
              │ Service  │    │ Service  │    │ Service  │
              └────┬─────┘    └────┬─────┘    └────┬─────┘
                   │               │               │
                   ▼               ▼               ▼
         ┌─────────────────────────────────────────────┐
         │              SystemPrompt Cloud             │
         │            systemprompt.io                  │
         └─────────────────────────────────────────────┘
```

## Contributing

Contributions are welcome! Please see our [Contributing Guidelines](CONTRIBUTING.md).

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

For more information, visit [systemprompt.io](https://systemprompt.io).

# README Implementation Instructions

Guidelines for creating the README.md for systemprompt-infrastructure crate.

## File Location

`/README.md` in repository root

## Structure

### 1. Title and Badges

```markdown
# systemprompt-infrastructure

[![Crates.io](https://img.shields.io/crates/v/systemprompt-infrastructure.svg)](https://crates.io/crates/systemprompt-infrastructure)
[![Documentation](https://docs.rs/systemprompt-infrastructure/badge.svg)](https://docs.rs/systemprompt-infrastructure)
[![CI](https://github.com/systempromptio/systemprompt-infrastructure/actions/workflows/ci.yaml/badge.svg)](https://github.com/systempromptio/systemprompt-infrastructure/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/systempromptio/systemprompt-infrastructure/branch/main/graph/badge.svg)](https://codecov.io/gh/systempromptio/systemprompt-infrastructure)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
```

### 2. One-Liner Description

A single sentence explaining what the crate does and why it exists.

Example:
> MCP server providing infrastructure tools for syncing, deploying, and managing SystemPrompt cloud deployments.

### 3. Links to Main Project

**IMPORTANT**: Always link to https://systemprompt.io for further information. This is the canonical source for documentation, guides, and support.

```markdown
## Part of the SystemPrompt Ecosystem

This crate is part of the [SystemPrompt](https://systemprompt.io) platform. For comprehensive documentation, tutorials, and support, visit **[systemprompt.io](https://systemprompt.io)**.

- [SystemPrompt](https://systemprompt.io) - Main project website
- [Documentation](https://systemprompt.io/docs) - Full documentation
- [Getting Started](https://systemprompt.io/docs/getting-started) - Quick start guide
- [systemprompt-core](https://github.com/systempromptio/systemprompt-core) - Core library
```

### 4. Features

List key capabilities with brief descriptions:

```markdown
## Features

- **Cloud Sync** - Bidirectional sync between local and cloud environments
- **One-Command Deploy** - Build and deploy to Fly.io with a single tool call
- **Export/Backup** - Export content and skills to disk
- **Status Monitoring** - Check deployment and connection status
- **Configuration Display** - View and filter current configuration
```

### 5. Installation

```markdown
## Installation

Add to your `Cargo.toml`:

\`\`\`toml
[dependencies]
systemprompt-infrastructure = "0.1"
\`\`\`

Or install via cargo:

\`\`\`bash
cargo add systemprompt-infrastructure
\`\`\`
```

### 6. Quick Start

Minimal working example to get started:

```markdown
## Quick Start

\`\`\`rust
use systemprompt_infrastructure::InfrastructureServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize your database connection
    let db_pool = systemprompt_infrastructure::create_database_connection().await?;

    // Create the server
    let server = InfrastructureServer::new(
        db_pool,
        "my-server".into(),
        app_context,
    );

    // Server is ready to handle MCP requests
    Ok(())
}
\`\`\`
```

### 7. MCP Configuration

Show how to add to Claude Desktop or other MCP clients:

```markdown
## MCP Configuration

### Claude Desktop

Add to your `claude_desktop_config.json`:

\`\`\`json
{
  "mcpServers": {
    "systemprompt-infrastructure": {
      "command": "systemprompt-infrastructure",
      "env": {
        "DATABASE_URL": "sqlite:///path/to/db.sqlite",
        "MCP_PORT": "5010"
      }
    }
  }
}
\`\`\`

### HTTP Transport

The server also supports HTTP transport on the configured port:

\`\`\`bash
MCP_PORT=5010 systemprompt-infrastructure
\`\`\`
```

### 8. Available Tools

Document each MCP tool:

```markdown
## Available Tools

| Tool | Description |
|------|-------------|
| `sync` | Sync files, database, content, skills, or all between local and cloud |
| `export` | Export content and skills to disk for backup |
| `deploy` | Build Rust binary, web assets, Docker image, and deploy to Fly.io |
| `status` | Get cloud connection state, deployment status, and configuration |
| `config` | Display current configuration by section (agents, mcp, skills, ai, web, content, env, settings) |

### Tool Examples

#### Sync

\`\`\`
sync(target: "all", direction: "push")
sync(target: "database", direction: "pull")
sync(target: "files")
\`\`\`

#### Deploy

\`\`\`
deploy()
\`\`\`

#### Status

\`\`\`
status()
\`\`\`
```

### 9. Environment Variables

```markdown
## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Database connection string | Required |
| `MCP_PORT` | HTTP server port | `5010` |
| `MCP_SERVICE_ID` | Service identifier | `systemprompt-infrastructure` |
```

### 10. Architecture Overview (Optional)

Brief description of how the system works:

```markdown
## Architecture

\`\`\`
┌─────────────────┐     ┌──────────────────┐
│  MCP Client     │────▶│  Infrastructure  │
│  (Claude, etc)  │     │  Server          │
└─────────────────┘     └────────┬─────────┘
                                 │
                    ┌────────────┼────────────┐
                    ▼            ▼            ▼
              ┌─────────┐  ┌─────────┐  ┌─────────┐
              │  Sync   │  │ Deploy  │  │ Export  │
              │ Service │  │ Service │  │ Service │
              └────┬────┘  └────┬────┘  └────┬────┘
                   │            │            │
                   ▼            ▼            ▼
              ┌─────────────────────────────────┐
              │         Cloud / Local           │
              └─────────────────────────────────┘
\`\`\`
```

### 11. Contributing

```markdown
## Contributing

Contributions are welcome! Please see our [Contributing Guidelines](CONTRIBUTING.md).

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
```

### 12. License

```markdown
## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
```

## Style Guidelines

1. **Be concise** - Developers scan READMEs, don't write essays
2. **Show, don't tell** - Code examples over lengthy explanations
3. **Start with value** - Lead with what the user gets, not implementation details
4. **Use tables** - For environment variables, tools, and options
5. **Keep examples minimal** - Show the simplest working case first
6. **Link, don't duplicate** - Reference docs.rs for API details

## Badges Order

Recommended badge order (left to right):
1. crates.io version
2. docs.rs
3. CI status
4. Coverage
5. License

## What NOT to Include

- Internal implementation details
- Changelog (use CHANGELOG.md)
- Detailed API documentation (use docs.rs)
- Development setup (use CONTRIBUTING.md)
- Issue templates

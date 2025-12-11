# SystemPrompt Template

A working skeleton for building your own SystemPrompt implementation.

## Quick Start

```bash
# 1. Start systemprompt-db (multi-tenant database service)
cd ../systemprompt-db
docker-compose -f docker-compose.local.yml up -d

# 2. Clone this template
git clone https://github.com/systempromptio/systemprompt-template my-project
cd my-project

# 3. Run setup (provisions database, builds, runs migrations)
just setup

# 4. Start the server
just start

# 5. Open http://localhost:8080
```

## Prerequisites

- **Rust** (1.75+) - https://rustup.rs/
- **Docker** - For running systemprompt-db
- **just** - Command runner: `cargo install just`
- **jq** - JSON processor: `sudo apt install jq`
- **systemprompt-db** - Multi-tenant PostgreSQL service (must be running)

## Project Structure

```
systemprompt-template/
├── core/                    # READ-ONLY - SystemPrompt Core (git subtree)
├── services/         # YOUR CODE - Customize here
│   ├── agents/              # Agent configurations (YAML)
│   ├── ai/                  # AI provider settings
│   ├── config/              # Root services config
│   ├── content/             # Markdown content (blog, legal pages)
│   ├── mcp/                 # MCP servers (Rust crates)
│   ├── skills/              # Agent skills
│   └── web/                 # Theme and branding
├── infrastructure/          # Docker, scripts
├── config/                  # Root configuration
└── justfile                 # Development commands
```

## Commands

| Command | Description |
|---------|-------------|
| `just setup` | First-time setup (provisions DB, builds, migrates) |
| `just build` | Build debug binaries |
| `just start` | Start API server |
| `just db-migrate` | Run migrations |
| `just core-sync` | Update core subtree |
| `just test` | Run tests |
| `just lint` | Run clippy |

## Configuration

### Environment Files

- `.env.secrets` - Database URL, API keys, JWT secret (gitignored)
- `.env.local` - Local development paths and settings

### Key Configuration Files

| File | Purpose |
|------|---------|
| `services/config/config.yml` | Root services config |
| `services/content/config.yml` | Content sources |
| `services/web/config.yml` | Theme and branding |
| `services/ai/config.yml` | AI providers |
| `config/ai.yaml` | Root AI config |

## Database Setup

This template uses **systemprompt-db** for multi-tenant database management.

The `just setup` command will:
1. Prompt for a tenant name (default: directory name)
2. Call the systemprompt-db API to provision an isolated database
3. Save the connection string to `.env.secrets`

If your tenant already exists, you'll be prompted to enter your existing DATABASE_URL.

### Manual Database Setup

If you need to provision manually:

```bash
# Create tenant via API
curl -X POST http://localhost:8085/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "mytenant"}'

# Save the returned connection_string to .env.secrets
```

## Core Submodule (READ-ONLY)

The `core/` directory contains SystemPrompt Core as a git submodule.

**Do not modify files in `core/`** - a pre-commit hook will prevent this.

To update core:
```bash
just core-sync    # Update submodule + Cargo deps
just core-version # Show current version
```

When cloning, use `--recursive` to fetch the submodule:
```bash
git clone --recursive https://github.com/systempromptio/systemprompt-template my-project
```

## Adding Your Own Code

### Add an MCP Server

1. Create a new crate in `services/mcp/your-server/`
2. Add a `module.yml` configuration
3. Add to `services/config/config.yml` includes
4. Add to `Cargo.toml` workspace members

### Add an Agent

1. Create `services/agents/your-agent.yml`
2. Add to `services/config/config.yml` includes

### Add Content

1. Add markdown files to `services/content/blog/`
2. Content is automatically indexed on startup

### Customize Theme

Edit `services/web/config.yml` to change:
- Branding (name, logo, colors)
- Typography
- Layout
- Navigation

## License

MIT License - see [LICENSE](LICENSE)

This project depends on [systemprompt-core](https://github.com/systempromptio/systemprompt-core) which is licensed under Apache 2.0.

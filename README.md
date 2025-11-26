# SystemPrompt Template

A working skeleton for building your own SystemPrompt implementation.

## Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/systempromptio/systemprompt-template my-project
cd my-project

# 2. Run setup (installs hooks, creates configs, starts DB, builds)
just setup

# 3. Edit your secrets
nano .env.secrets  # Add DATABASE_URL, JWT_SECRET, etc.

# 4. Start the server
just start

# 5. Open http://localhost:8080
```

## Prerequisites

- **Rust** (1.75+) - https://rustup.rs/
- **Docker** - For PostgreSQL
- **just** - Command runner: `cargo install just`

## Project Structure

```
systemprompt-template/
├── core/                    # READ-ONLY - SystemPrompt Core (git subtree)
├── crates/services/         # YOUR CODE - Customize here
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
| `just setup` | First-time setup |
| `just build` | Build debug binaries |
| `just start` | Start API server |
| `just db-up` | Start PostgreSQL |
| `just db-migrate` | Run migrations |
| `just core-sync` | Update core subtree |
| `just test` | Run tests |
| `just lint` | Run clippy |

## Configuration

### Environment Files

- `.env.secrets` - Database, API keys, JWT secret (gitignored)
- `.env.local` - Local development paths and settings

### Key Configuration Files

| File | Purpose |
|------|---------|
| `crates/services/config/config.yml` | Root services config |
| `crates/services/content/config.yml` | Content sources |
| `crates/services/web/config.yml` | Theme and branding |
| `crates/services/ai/config.yml` | AI providers |
| `config/ai.yaml` | Root AI config |

## Core Subtree (READ-ONLY)

The `core/` directory contains SystemPrompt Core as a git subtree.

**Do not modify files in `core/`** - a pre-commit hook will prevent this.

To update core:
```bash
just core-sync
```

## Adding Your Own Code

### Add an MCP Server

1. Create a new crate in `crates/services/mcp/your-server/`
2. Add a `module.yml` configuration
3. Add to `crates/services/config/config.yml` includes
4. Add to `Cargo.toml` workspace members

### Add an Agent

1. Create `crates/services/agents/your-agent.yml`
2. Add to `crates/services/config/config.yml` includes

### Add Content

1. Add markdown files to `crates/services/content/blog/`
2. Content is automatically indexed on startup

### Customize Theme

Edit `crates/services/web/config.yml` to change:
- Branding (name, logo, colors)
- Typography
- Layout
- Navigation

## License

MIT

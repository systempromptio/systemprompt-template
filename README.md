# SystemPrompt Template

A working skeleton for building your own SystemPrompt implementation.

## Quick Start

### 1. Create Your Project

**Option A: GitHub CLI**
```bash
gh repo create my-project --template systempromptio/systemprompt-template --clone --private
cd my-project
git submodule update --init --recursive
```

**Option B: Web UI**

Click **"Use this template"** on [systemprompt-template](https://github.com/systempromptio/systemprompt-template), then:
```bash
git clone --recursive https://github.com/YOUR_USERNAME/my-project.git
cd my-project
```

### 2. Build the CLI

```bash
# Build (first time requires offline mode - no database yet)
SQLX_OFFLINE=true cargo build --release -p systemprompt-cli
```

### 3. Setup Your Environment

All configuration is done through the CLI. Choose your database option:

#### Option A: Local PostgreSQL (Free)

```bash
# Start PostgreSQL in Docker
docker run -d --name systemprompt-db \
  -e POSTGRES_DB=systemprompt \
  -e POSTGRES_USER=systemprompt \
  -e POSTGRES_PASSWORD=systemprompt \
  -p 5432:5432 \
  postgres:16

# Login to systemprompt.io Cloud
systemprompt cloud auth login

# Create a local tenant (will prompt for database URL)
systemprompt cloud tenant create --type local

# Create your profile
systemprompt cloud profile create local

# Run database migrations
systemprompt infra db migrate

# Start the server
systemprompt infra services start --all
```

#### Option B: Cloud PostgreSQL (Paid)

```bash
# Login to systemprompt.io Cloud
systemprompt cloud auth login

# Create a cloud tenant (provisions managed PostgreSQL)
systemprompt cloud tenant create --region iad

# Create your profile
systemprompt cloud profile create production

# Run database migrations
systemprompt infra db migrate

# Start the server
systemprompt infra services start --all
```

The API server will be available at `http://127.0.0.1:8080`.

## Prerequisites

- **Rust** (1.75+) - https://rustup.rs/
- **Docker** - For local PostgreSQL **OR** systemprompt.io Cloud account

## Project Structure

```
systemprompt-template/
├── core/                    # READ-ONLY - SystemPrompt Core (git submodule)
├── extensions/              # ALL Rust code
│   ├── blog/                # Reference blog extension
│   └── mcp/                 # MCP servers
│       └── systemprompt/  # MCP server
├── services/                # YOUR CONFIG - Customize here
│   ├── agents/              # Agent configurations (YAML)
│   ├── ai/                  # AI provider settings
│   ├── config/              # Root services config
│   ├── content/             # Markdown content (blog, legal pages)
│   ├── mcp/                 # MCP server configurations (YAML)
│   ├── skills/              # Agent skills
│   └── web/                 # Theme and branding
└── justfile                 # Development commands
```

## Commands

### Development

| Command | Description |
|---------|-------------|
| `systemprompt infra services start --all` | Start all services (API, agents, MCP) |
| `systemprompt infra services status` | Check service status |
| `systemprompt infra db migrate` | Run database migrations |
| `systemprompt infra db status` | Check database connection |
| `systemprompt admin agents list` | List configured agents |
| `systemprompt admin agents status <name>` | Check agent status |

### Build & Deploy

| Command | Description |
|---------|-------------|
| `cargo build --release -p systemprompt-cli` | Build release binary |
| `systemprompt cloud auth login` | Authenticate with SystemPrompt Cloud |
| `systemprompt cloud tenant create` | Create a tenant (local or cloud) |
| `systemprompt cloud profile create <name>` | Create a profile |
| `systemprompt cloud deploy` | Deploy to cloud |

## Configuration

### Environment Files

- `.env.secrets` - Database URL, API keys, JWT secret (gitignored)
- `.env.local` - Local development paths and settings

### Key Configuration Files

| File | Purpose |
|------|---------|
| `services/config/config.yaml` | Root services config |
| `services/content/config.yaml` | Content sources |
| `services/web/config.yaml` | Theme and branding |
| `services/ai/config.yaml` | AI providers |

## Database Setup

Database configuration is managed through the CLI using tenants and profiles.

### Local PostgreSQL (Free)

```bash
# Start PostgreSQL in Docker
docker run -d --name systemprompt-db \
  -e POSTGRES_DB=systemprompt \
  -e POSTGRES_USER=systemprompt \
  -e POSTGRES_PASSWORD=systemprompt \
  -p 5432:5432 \
  postgres:16

# Create a local tenant (will prompt for database URL)
systemprompt cloud tenant create --type local

# Connection URL: postgresql://systemprompt:systemprompt@localhost:5432/systemprompt
```

### Cloud PostgreSQL (Paid)

```bash
# Create a cloud tenant (auto-provisions managed PostgreSQL)
systemprompt cloud tenant create --region iad
```

### Checking Database Status

```bash
systemprompt infra db status
systemprompt infra db tables
```

## Core Submodule (READ-ONLY)

The `core/` directory contains SystemPrompt Core as a git submodule.

**Do not modify files in `core/`** - a pre-commit hook will prevent this.

To update core:
```bash
git submodule update --remote core
cargo update
```

When cloning, always use `--recursive` to fetch the submodule:
```bash
git clone --recursive https://github.com/systempromptio/systemprompt-template my-project
```

## Adding Your Own Code

### Add an MCP Server

1. Create a new crate in `extensions/mcp/your-server/`
2. Add a `manifest.yaml` configuration
3. Add a YAML config in `services/mcp/your-server.yaml`
4. Add to `services/config/config.yaml` includes
5. Add to `Cargo.toml` workspace excludes (built separately)

### Add an Agent

1. Create `services/agents/your-agent.yaml`
2. Add to `services/config/config.yaml` includes

### Add Content

1. Add markdown files to `services/content/blog/`
2. Content is automatically indexed on startup

### Customize Theme

Edit `services/web/config.yaml` to change:
- Branding (name, logo, colors)
- Typography
- Layout
- Navigation

## Deploying to SystemPrompt Cloud

### Quick Deploy

```bash
# 1. Login to SystemPrompt Cloud
systemprompt cloud auth login

# 2. Create a cloud tenant (first time only)
systemprompt cloud tenant create --region iad

# 3. Create a production profile
systemprompt cloud profile create production

# 4. Build release binary
cargo build --release -p systemprompt-cli

# 5. Deploy
systemprompt cloud deploy --profile production
```

### Deploy Options

| Flag | Description |
|------|-------------|
| `--profile <NAME>` | Profile to deploy |
| `--skip-push` | Build Docker image but don't push to registry |

### Checking Deployment Status

```bash
systemprompt cloud auth whoami
systemprompt cloud tenant list
systemprompt cloud tenant show
```

## License

MIT License - see [LICENSE](LICENSE)

This project depends on [systemprompt-core](https://github.com/systempromptio/systemprompt-core) which is licensed under FSL-1.1-ALv2.

# SystemPrompt Template

A working skeleton for building your own SystemPrompt implementation.

## Quick Start

### 1. Create Your Project

**Option A: CLI**
```bash
gh repo create my-project --template systempromptio/systemprompt-template --clone
cd my-project
git submodule update --init --recursive
```

**Option B: Web UI**

Click **"Use this template"** on [systemprompt-template](https://github.com/systempromptio/systemprompt-template), then:
```bash
git clone https://github.com/YOUR_USERNAME/my-project.git
cd my-project
git submodule update --init --recursive
```

### 2. Build & Configure

```bash
# Build (first time requires offline mode - no database yet)
SQLX_OFFLINE=true just build

# Authenticate with SystemPrompt Cloud
just login

# Create local tenant (provisions database)
just tenant
# → Select "Create new tenant" → Choose "local"

# (Optional) Create cloud tenant for deployment
just tenant
# → Select "Create new tenant" → Choose "cloud"

# Configure profile
just profile
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

### Development

| Command | Description |
|---------|-------------|
| `just setup` | First-time setup (provisions DB, builds, migrates) |
| `just build` | Build debug binaries |
| `just start` | Start API server |
| `just db-migrate` | Run migrations |
| `just core-sync` | Update core subtree |
| `just test` | Run tests |
| `just lint` | Run clippy |

### Build & Deploy

| Command | Description |
|---------|-------------|
| `systemprompt build release` | Build optimized binary + web assets |
| `systemprompt build release --skip-web` | Build binary only (faster) |
| `systemprompt cloud login` | Authenticate with SystemPrompt Cloud |
| `systemprompt cloud setup` | Link project to cloud tenant |
| `systemprompt cloud deploy` | Deploy pre-built artifacts |
| `systemprompt cloud deploy --rebuild` | Rebuild and deploy |

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

## Deploying to SystemPrompt Cloud

### Quick Deploy

```bash
# 1. Login to SystemPrompt Cloud
systemprompt cloud login

# 2. Link your project to a cloud tenant (first time only)
systemprompt cloud setup --name my-project

# 3. Build release artifacts
systemprompt build release

# 4. Deploy
systemprompt cloud deploy
```

### Build & Release Architecture

The build system uses a **type-safe pipeline** that enforces correct build order at compile time:

```
Unbuilt → BinaryReady → Complete
```

#### Build Commands

| Command | Description |
|---------|-------------|
| `systemprompt build release` | Build release binary + web assets, stage for deployment |
| `systemprompt build release --skip-web` | Build only the binary (faster, no web assets) |
| `systemprompt cloud deploy` | Deploy using pre-built artifacts |
| `systemprompt cloud deploy --rebuild` | Rebuild everything before deploying |

#### Workflow: Separate Build & Deploy (Recommended)

```bash
# Step 1: Build once
systemprompt build release

# Step 2: Deploy (fast - uses pre-built artifacts)
systemprompt cloud deploy

# Subsequent deploys reuse artifacts until you rebuild
systemprompt cloud deploy --tag v1.0.0
```

#### Workflow: All-in-One (Convenience)

```bash
# Build + deploy in one command
systemprompt cloud deploy --rebuild
```

### Artifacts

After `systemprompt build release`, artifacts are staged to:

```
infrastructure/build-context/release/
├── systemprompt    # Release binary (optimized, stripped)
web/dist/           # Static web assets
```

### Deploy Options

| Flag | Description |
|------|-------------|
| `--rebuild` | Rebuild all artifacts before deploying |
| `--skip-push` | Build Docker image but don't push to registry |
| `--tag <TAG>` | Custom image tag (default: `deploy-{timestamp}-{git_sha}`) |

### Breaking Change Note

The default behavior changed from "always rebuild" to "use pre-built artifacts":

- **Before**: `cloud deploy` rebuilt everything by default
- **After**: `cloud deploy` expects pre-built artifacts; use `--rebuild` to rebuild

This separation enables faster iteration - build once, deploy many times.

## License

MIT License - see [LICENSE](LICENSE)

This project depends on [systemprompt-core](https://github.com/systempromptio/systemprompt-core) which is licensed under Apache 2.0.

# SystemPrompt Template - AI Assistant Instructions

## Critical Rules

1. **NEVER modify files in `core/`** - This is a READ-ONLY git submodule from systemprompt-core
2. **ALWAYS use just commands** - See `just --list` for available commands
3. **NEVER commit secrets** - Keep `.env.secrets` out of version control

## Project Structure

```
systemprompt-template/
├── core/                    # READ-ONLY - Do not modify
├── services/         # Implementation code - Edit here
│   ├── agents/              # Agent YAML configs
│   ├── ai/                  # AI provider config
│   ├── config/              # Root services config
│   ├── content/             # Markdown content
│   ├── mcp/                 # MCP servers (Rust)
│   ├── skills/              # Agent skills
│   └── web/                 # Theme config
├── infrastructure/          # Docker, scripts
├── config/                  # Root config files
└── justfile                 # Commands
```

## Common Tasks

### Build and Run
```bash
just build    # Build debug binaries
just start    # Start API server
```

### Database
```bash
just db-up       # Start PostgreSQL
just db-migrate  # Run migrations
```

### Update Core
```bash
just core-sync      # Update submodule + Cargo deps
just core-update    # Update submodule only
just core-version   # Show current version
just core-pin v0.1.0  # Pin to specific version
```

### Clone with Core
```bash
git clone --recursive <repo-url>
# or after clone:
git submodule update --init --recursive
```

## Configuration Files

- `services/config/config.yml` - Services configuration
- `services/content/config.yml` - Content sources
- `services/web/config.yml` - Theme and branding
- `config/ai.yaml` - AI providers

## Adding Features

### New MCP Server
1. Create crate in `services/mcp/your-server/`
2. Add `module.yml` with server config
3. Add to workspace in `Cargo.toml`
4. Include in `services/config/config.yml`

### New Agent
1. Create `services/agents/your-agent.yml`
2. Include in `services/config/config.yml`

### New Content
Add markdown files to `services/content/blog/your-post/index.md`

## Environment Variables

Required (in `.env.secrets`):
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - 64+ character secret
- `ADMIN_PASSWORD` - Admin access password

Optional:
- `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GEMINI_API_KEY` - AI providers

## File Naming Conventions

- Config files: `config.yml`
- Agent configs: `{name}.yml`
- MCP servers: Directory name matches crate name
- Content: `{slug}/index.md`

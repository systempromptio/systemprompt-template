# SystemPrompt Template - AI Assistant Instructions

## Critical Rules

1. **NEVER modify files in `core/`** - This is a READ-ONLY git subtree from systemprompt-core
2. **ALWAYS use just commands** - See `just --list` for available commands
3. **NEVER commit secrets** - Keep `.env.secrets` out of version control

## Project Structure

```
systemprompt-template/
├── core/                    # READ-ONLY - Do not modify
├── crates/services/         # Implementation code - Edit here
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
just core-sync   # Pull latest from systemprompt-core
```

## Configuration Files

- `crates/services/config/config.yml` - Services configuration
- `crates/services/content/config.yml` - Content sources
- `crates/services/web/config.yml` - Theme and branding
- `config/ai.yaml` - AI providers

## Adding Features

### New MCP Server
1. Create crate in `crates/services/mcp/your-server/`
2. Add `module.yml` with server config
3. Add to workspace in `Cargo.toml`
4. Include in `crates/services/config/config.yml`

### New Agent
1. Create `crates/services/agents/your-agent.yml`
2. Include in `crates/services/config/config.yml`

### New Content
Add markdown files to `crates/services/content/blog/your-post/index.md`

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

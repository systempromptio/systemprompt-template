# Services Directory

This directory contains all customizable service configurations for your SystemPrompt implementation. The `core/` submodule is read-only - all customizations happen here.

## Directory Structure

```
services/
├── config/config.yml     # Main configuration (aggregates all services)
├── agents/               # Agent definitions
├── ai/                   # AI provider configuration
├── content/              # Markdown content (blog, legal, etc.)
├── mcp/                  # MCP server configurations
├── scheduler/            # Background jobs configuration
├── skills/               # Agent skills
└── web/                  # Theme, branding, UI settings
```

## Configuration Flow

All configurations are aggregated through `services/config/config.yml`:

```yaml
includes:
  - ../agents/assistant.yml
  - ../agents/admin.yml
  - ../mcp/system-tools.yml
  - ../skills/config.yml
  - ../ai/config.yml
  - ../web/config.yml
  - ../scheduler/config.yml
```

To enable a new agent or MCP server, add it to the `includes` list.

---

## Agents

Location: `services/agents/`

Agents are AI assistants with specific capabilities. Each agent has:
- A unique port in the range 9000-9999
- An A2A (Agent-to-Agent) card for discovery
- A system prompt defining behavior
- Optional MCP server connections

### Agent Configuration Schema

```yaml
agents:
  your-agent:
    name: "your-agent"
    port: 9001                    # Unique port (9000-9999)
    enabled: true
    is_primary: false
    default: false
    endpoint: "/api/v1/agents/your-agent"
    card:
      protocolVersion: "0.3.0"
      name: "Your Agent"
      displayName: "Your Agent"
      description: "What this agent does"
      version: "1.0.0"
      preferredTransport: "JSONRPC"
      provider:
        organization: "your-org"
        url: "https://example.com"
      iconUrl: "https://ui-avatars.com/api/?name=YA&background=3b82f6&color=fff"
      capabilities:
        streaming: true
        pushNotifications: false
        stateTransitionHistory: false
      defaultInputModes: ["text/plain"]
      defaultOutputModes: ["text/plain", "application/json"]
      securitySchemes:
        oauth2:
          type: oauth2
          flows:
            authorizationCode:
              authorizationUrl: "/api/v1/core/oauth/authorize"
              tokenUrl: "/api/v1/core/oauth/token"
              scopes:
                user: "Authenticated user access"
      security:
        - oauth2: ["user"]
      skills: []
    metadata:
      systemPrompt: |
        Your agent's instructions here.
      mcpServers:
        - system-tools    # Reference MCP servers by name
```

### Creating a New Agent

1. Create `services/agents/your-agent.yml`
2. Add to `services/config/config.yml`:
   ```yaml
   includes:
     - ../agents/your-agent.yml
   ```
3. Restart the server

### Existing Agents

| Agent | Port | Description |
|-------|------|-------------|
| assistant | 9000 | Primary user-facing assistant |
| admin | 9001 | Administrative operations |
| agent-editor | 9002 | Creates/modifies agent configs |
| skill-editor | 9003 | Creates/modifies skills |
| infrastructure | 9004 | Deployment and sync management |

---

## MCP Servers

Location: `services/mcp/`

MCP (Model Context Protocol) servers provide tools to agents. Each server:
- Runs on a unique port (5000-5999)
- Exposes tools via the MCP protocol
- Can require OAuth authentication

### MCP Server Configuration Schema

```yaml
mcp_servers:
  your-server:
    binary: "your-server"           # Binary name
    package: "your-server"          # Cargo package name
    path: "services/mcp/your-server"
    port: 5004                      # Unique port (5000-5999)
    endpoint: "http://localhost:8080/api/v1/mcp/your-server/mcp"
    enabled: true                   # Set to true to start
    display_in_web: true
    schemas: []
    oauth:
      required: true
      scopes: ["user"]
      audience: "mcp"
      client_id: null
    description: "What this MCP server provides"
```

### Why MCP Servers Might Not Start

Check these common issues:

1. **`enabled: false`** - The server is disabled in its config
2. **Compile errors** - The Rust binary failed to build
3. **Port conflict** - Another service is using the port
4. **Missing from includes** - Not listed in `config/config.yml`

### Enabling an MCP Server

Edit the server's YAML file and set `enabled: true`:

```yaml
mcp_servers:
  system-tools:
    enabled: true  # Change from false to true
```

### Creating a New MCP Server

1. Create directory `services/mcp/your-server/`
2. Add Rust crate with MCP implementation
3. Create `services/mcp/your-server.yml` config
4. Add to workspace `Cargo.toml`
5. Add to `services/config/config.yml`
6. Build: `just build`

---

## Skills

Location: `services/skills/`

Skills define reusable capabilities that agents can use. Each skill contains:
- Markdown instructions
- Tool definitions
- Validation schemas

### Skill Structure

```
services/skills/
├── config.yml                    # Aggregates all skills
├── your_skill/
│   ├── config.yml               # Skill definition
│   ├── instructions.md          # Markdown instructions
│   └── tools/                   # Tool definitions (optional)
```

### Skill Configuration Schema

```yaml
skills:
  your_skill:
    name: "Your Skill"
    description: "What this skill does"
    enabled: true
    instructions_file: "instructions.md"
    tools:
      - name: "your_tool"
        description: "What this tool does"
        input_schema:
          type: object
          properties:
            param1:
              type: string
              description: "Parameter description"
          required: ["param1"]
```

### Adding a New Skill

1. Create `services/skills/your_skill/config.yml`
2. Create `services/skills/your_skill/instructions.md`
3. Add to `services/skills/config.yml`:
   ```yaml
   includes:
     - your_skill/config.yml
   ```

### Existing Skills

| Skill | Description |
|-------|-------------|
| agent_creation | Create new agents |
| agent_editing | Modify existing agents |
| skill_creation | Create new skills |
| skill_editing | Modify existing skills |
| deployment | Deploy configurations |
| sync_management | Sync with core |
| file_context_reasoning | AI-powered file analysis |

---

## AI Configuration

Location: `services/ai/config.yml`

Configure AI providers and MCP settings:

```yaml
ai:
  default_provider: gemini
  default_max_output_tokens: 8192
  providers:
    anthropic:
      enabled: true
      api_key: ${ANTHROPIC_API_KEY}
      default_model: claude-sonnet-4-20250514
    openai:
      enabled: false
      api_key: ${OPENAI_API_KEY}
      default_model: gpt-4-turbo
    gemini:
      enabled: true
      api_key: ${GEMINI_API_KEY}
      default_model: gemini-2.5-flash
  mcp:
    auto_discover: true
    connect_timeout_ms: 5000
    execution_timeout_ms: 30000
```

---

## Web Configuration

Location: `services/web/config.yml`

Configure theme, branding, and UI:
- Branding (name, logo, colors)
- Typography (fonts, sizes)
- Layout (spacing, shadows)
- Navigation (footer links, social)

---

## Scheduler

Location: `services/scheduler/config.yml`

Configure background jobs:

```yaml
scheduler:
  enabled: true
  jobs:
    - name: cleanup_anonymous_users
      enabled: true
      schedule: "0 0 3 * * *"    # Cron format
```

---

## Content

Location: `services/content/`

Markdown content for your site:
- `blog/` - Blog articles
- `legal/` - Legal pages (privacy policy, etc.)

Content structure:
```
services/content/
├── config.yml           # Content source definitions
├── blog/
│   └── your-post/
│       └── index.md     # Article content
└── legal/
    └── privacy-policy/
        └── index.md
```

---

## Troubleshooting

### MCP Servers Not Starting

1. Check if enabled:
   ```bash
   grep -r "enabled:" services/mcp/*.yml
   ```

2. Check for compile errors:
   ```bash
   just build
   ```

3. Check if included in config:
   ```bash
   grep "mcp/" services/config/config.yml
   ```

### Agent Not Appearing

1. Verify the agent file is included in `services/config/config.yml`
2. Check `enabled: true` in the agent config
3. Ensure unique port number

### Changes Not Taking Effect

1. Restart the server: `just start`
2. Check for YAML syntax errors
3. Verify the file is included in the main config

---

## Quick Reference

| Service Type | Location | Port Range | Config Key |
|-------------|----------|------------|------------|
| Agents | `services/agents/` | 9000-9999 | `agents` |
| MCP Servers | `services/mcp/` | 5000-5999 | `mcp_servers` |
| Skills | `services/skills/` | N/A | `skills` |
| AI | `services/ai/` | N/A | `ai` |
| Web | `services/web/` | N/A | `branding`, `colors`, etc. |
| Scheduler | `services/scheduler/` | N/A | `scheduler` |

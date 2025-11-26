# Agent Templates

This directory contains example agent configurations for SystemPrompt OS.

## Structure

```
templates/agents/
├── README.md                    # This file
├── agents.yaml                  # Multi-agent configuration example
├── basic-agent.yaml            # Simple agent template
├── mcp-enabled-agent.yaml      # Agent with MCP server integration
└── streaming-agent.yaml        # Agent with streaming capabilities
```

## Usage

### 1. Basic Agent Template

Copy a template to create a new agent:

```bash
cp templates/agents/basic-agent.yaml infrastructure/configs/my-agent.yaml
```

### 2. Configure Agent

Edit the configuration:

```yaml
agents:
  my-agent:
    uuid: "<generate-unique-uuid>"
    name: "my-agent"
    port: 9001
    enabled: true
    card:
      displayName: "My Custom Agent"
      description: "Description of what this agent does"
      version: "1.0.0"
    metadata:
      systemPrompt: |
        You are My Custom Agent.
        Your purpose is to...
```

### 3. Deploy Agent

Use the CLI to deploy:

```bash
just a2a deploy --config infrastructure/configs/my-agent.yaml
```

## Template Descriptions

### `basic-agent.yaml`
Minimal agent configuration with text-only I/O.

**Use cases:**
- Simple conversational agents
- Text processing assistants
- Quick prototypes

### `mcp-enabled-agent.yaml`
Agent integrated with MCP servers for extended capabilities.

**Use cases:**
- Administrative agents with system access
- Agents requiring external tools (filesystem, GitHub, etc.)
- Multi-tool orchestration

### `streaming-agent.yaml`
Agent with streaming response capabilities.

**Use cases:**
- Real-time conversation agents
- Progressive content generation
- Long-running tasks with live updates

### `agents.yaml`
Multi-agent configuration example (current production config).

**Use cases:**
- Production deployments
- Multiple coordinated agents
- Service orchestration

## Configuration Reference

### Required Fields

```yaml
agents:
  <agent-id>:
    uuid: string              # Unique identifier (UUID v4)
    name: string              # Agent name (lowercase, hyphen-separated)
    port: number              # Port (9000-9999 range)
    enabled: boolean          # Auto-start with API
    card:                     # A2A Protocol card
      displayName: string
      description: string
      version: string
      protocolVersion: "0.3.0"
```

### Optional Fields

```yaml
    is_primary: boolean       # Mark as primary agent (default: false)
    metadata:
      systemPrompt: string    # Agent system prompt
      mcpServers: [string]    # List of MCP server names to load
    oauth:
      required: boolean       # Require OAuth authentication
      scopes: [string]        # Required OAuth scopes
      audience: string        # JWT audience (default: "a2a")
    card:
      capabilities:
        streaming: boolean
        pushNotifications: boolean
        stateTransitionHistory: boolean
      defaultInputModes: [string]
      defaultOutputModes: [string]
```

## Best Practices

1. **Use unique UUIDs** - Generate with `uuidgen` or online tools
2. **Assign unique ports** - Check existing agents to avoid conflicts
3. **Clear system prompts** - Define agent purpose and behavior explicitly
4. **Enable only when needed** - Set `enabled: false` for inactive agents
5. **Version semantically** - Use semantic versioning (1.0.0, 1.1.0, etc.)

## Examples

### Creating a Research Agent

```yaml
agents:
  research-agent:
    uuid: "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
    name: "research-agent"
    port: 9002
    enabled: true
    card:
      displayName: "Research Assistant"
      description: "Agent specialized in research and information gathering"
      version: "1.0.0"
      protocolVersion: "0.3.0"
      capabilities:
        streaming: true
      defaultInputModes:
        - "text/plain"
      defaultOutputModes:
        - "text/plain"
        - "text/markdown"
    metadata:
      systemPrompt: |
        You are a Research Assistant.
        Your role is to gather, analyze, and synthesize information.
        Provide well-researched, cited responses.
      mcpServers:
        - "brave-search"
        - "filesystem"
    oauth:
      required: false
      audience: "a2a"
```

### Creating a Code Assistant

```yaml
agents:
  code-agent:
    uuid: "b2c3d4e5-f6a7-8901-bcde-f23456789012"
    name: "code-agent"
    port: 9003
    enabled: true
    card:
      displayName: "Code Assistant"
      description: "Agent specialized in code generation and review"
      version: "1.0.0"
      protocolVersion: "0.3.0"
      capabilities:
        streaming: true
        stateTransitionHistory: true
    metadata:
      systemPrompt: |
        You are a Code Assistant.
        Help with code generation, review, debugging, and optimization.
        Follow best practices and write clean, documented code.
      mcpServers:
        - "filesystem"
        - "github"
    oauth:
      required: true
      scopes: ["admin"]
      audience: "a2a"
```

## Troubleshooting

**Port conflicts:**
```bash
# Check which ports are in use
just db query "SELECT name, port, status FROM services WHERE service_type = 'agent'"
```

**Agent not starting:**
```bash
# Check agent status
just a2a list

# View logs
just log
```

**MCP server not loading:**
```bash
# Verify MCP server exists
just mcp status

# Check agent metadata
just db query "SELECT name, mcp_servers FROM agent_metadata WHERE name = 'your-agent'"
```

## See Also

- `/templates/mcp/` - MCP server templates
- `/plan/agent/` - Agent architecture documentation
- `CLAUDE.md` - Service architecture overview

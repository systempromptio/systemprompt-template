# Services Configuration: Agents, Skills, and MCP Servers

This document explains the relationship between agents, skills, and MCP servers in SystemPrompt service configurations.

---

## Overview

SystemPrompt uses YAML configuration files to define three interconnected components:

| Component | Location | Purpose |
|-----------|----------|---------|
| **Agents** | `services/agents/*.yml` | AI agents that handle user interactions |
| **Skills** | `services/skills/*/config.yml` | Instruction sets that guide agent behavior |
| **MCP Servers** | `services/mcp/*.yml` | Tool providers that agents call for actions |

---

## Configuration Hierarchy

```
services/
├── config/
│   └── config.yml          → Master config (includes all others)
├── agents/
│   ├── edward.yml          → Primary agent
│   ├── content.yml         → Content agent
│   └── admin.yml           → Admin agent
├── skills/
│   ├── config.yml          → Skills master (includes all)
│   ├── linkedin_post_writing/
│   │   ├── config.yml      → Skill metadata
│   │   └── SKILL.md        → Skill instructions
│   └── research_blog/
│       ├── config.yml
│       └── index.md
├── mcp/
│   ├── content-manager.yml → Content MCP server
│   ├── systemprompt-admin.yml
│   └── tyingshoelaces.yml
└── ai/
    └── config.yml          → Default AI model configuration
```

---

## Agent Configuration

### Location

`services/agents/{agent-name}.yml`

### Structure

```yaml
agents:
  content:                              # Agent identifier (unique)
    name: "content"                     # Display name
    port: 9001                          # Unique port (9000-9999)
    endpoint: "/api/v1/agents/content"  # API endpoint
    enabled: true
    is_primary: false                   # Only one agent can be primary
    default: false                      # Only one agent can be default

    card:                               # A2A protocol metadata
      protocolVersion: "0.3.0"
      name: "Content"
      displayName: "Content Marketing Agent"
      description: "Creates blog posts and social content"
      version: "1.0.0"

      capabilities:
        streaming: true
        pushNotifications: true
        stateTransitionHistory: true

      security:                         # OAuth requirements
        - oauth2: ["user", "admin"]

      skills:                           # Skills this agent uses
        - id: "linkedin_post_writing"
          name: "LinkedIn Post Writing"
          description: "Create professional LinkedIn posts"
          tags: ["linkedin", "social-media"]
          examples:
            - "Create a LinkedIn post about AI"

        - id: "research_blog"
          name: "Research Blog"
          description: "Research topics for blog content"
          tags: ["research", "content"]

    metadata:
      provider: "gemini"                # AI provider override
      model: "gemini-2.5-flash"         # Model override

      systemPrompt: |
        # Content Marketing Agent

        ## Tools Available
        - `research_blog` - Research to validate insights
        - `create_blog_post` - Generate blog posts
        - `generate_social_content` - Create social posts

      mcpServers:                       # MCP servers this agent uses
        - "content-manager"
```

### Key Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Unique agent identifier |
| `port` | YES | Unique port in 9000-9999 range |
| `enabled` | YES | Whether agent is active |
| `card.skills` | NO | Skills exposed to users |
| `metadata.systemPrompt` | YES | Agent instructions |
| `metadata.mcpServers` | NO | MCP servers agent can call |

---

## Skill Configuration

### Location

`services/skills/{skill-id}/config.yml`

### Structure

```yaml
id: linkedin_post_writing               # Unique identifier (snake_case)
name: "LinkedIn Post Writing"           # Display name
description: "Create professional LinkedIn posts in Edward's voice"
enabled: true
version: "1.0.0"
file: "SKILL.md"                        # Instruction file (index.md or SKILL.md)

assigned_agents:                        # Agents that use this skill
  - content
  - marketing

tags:
  - social_media
  - linkedin
  - content_creation
```

### Skill Instruction File

`services/skills/{skill-id}/index.md` or `SKILL.md`:

```markdown
# LinkedIn Post Writing Skill

## Overview
Create engaging LinkedIn posts that drive professional engagement.

## Guidelines
1. Hook in first line
2. Clear structure with line breaks
3. End with call-to-action

## Output Format
- 1300 characters max
- Use emojis sparingly
- Include relevant hashtags

## Examples
[Include example posts]
```

### Skills Master Config

`services/skills/config.yml`:

```yaml
includes:
  - linkedin_post_writing/config.yml
  - research_blog/config.yml
  - traffic_analytics/config.yml

skills: {}                              # Aggregated from includes
```

---

## MCP Server Configuration

### Location

`services/mcp/{server-name}.yml`

### Structure

```yaml
mcp_servers:
  content-manager:                      # Server identifier
    binary: "content-manager"           # Executable name
    package: "content-manager"          # Package name
    path: "services/mcp/content-manager" # Local path
    port: 5003                          # Unique port (5000-5999)
    endpoint: "http://localhost:8080/api/v1/mcp/content-manager/mcp"
    enabled: true
    display_in_web: true

    oauth:                              # Authentication
      required: true
      scopes: ["user", "admin"]         # Required OAuth scopes
      audience: "mcp"
      client_id: null

    description: "Content Manager MCP Server - Create and manage blog content"

    model_config:                       # Default AI model for tools
      provider: gemini
      model: gemini-2.5-flash
      max_output_tokens: 16384

    tools:                              # Tool-specific configs
      create_blog_post:
        model_config:                   # Override for this tool
          provider: gemini
          model: gemini-3-pro-preview
          max_output_tokens: 32768
          thinking_level: low

      update_blog_post:
        model_config:
          provider: gemini
          model: gemini-3-pro-preview
          max_output_tokens: 32768

      delete_blog_post: {}              # Use default config

      generate_social_content:
        model_config:
          provider: gemini
          model: gemini-3-pro-preview
          max_output_tokens: 32768
```

### Key Fields

| Field | Required | Description |
|-------|----------|-------------|
| `binary` | YES | Executable binary name |
| `port` | YES | Unique port in 5000-5999 range |
| `enabled` | YES | Whether server is active |
| `oauth.required` | NO | Whether authentication needed |
| `oauth.scopes` | NO | Required OAuth scopes |
| `model_config` | NO | Default AI model for tools |
| `tools` | NO | Per-tool configuration |

---

## Relationship Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      AGENT (content.yml)                    │
│                                                             │
│  metadata:                                                  │
│    mcpServers: ["content-manager"]  ─────────────┐         │
│                                                   │         │
│  card:                                            │         │
│    skills:                                        │         │
│      - id: "linkedin_post_writing" ───────┐      │         │
│      - id: "research_blog" ───────────┐   │      │         │
│                                       │   │      │         │
└───────────────────────────────────────│───│──────│─────────┘
                                        │   │      │
                 ┌──────────────────────┘   │      │
                 │   ┌──────────────────────┘      │
                 ▼   ▼                             ▼
┌────────────────────────────────┐    ┌──────────────────────────┐
│ SKILL (linkedin_post_writing/) │    │ MCP SERVER (content-mgr) │
│                                │    │                          │
│  assigned_agents:              │    │  tools:                  │
│    - content ◄─────────────────│────│    - create_blog_post    │
│                                │    │    - update_blog_post    │
│  file: SKILL.md                │    │    - generate_social     │
│  tags: [linkedin, social]      │    │    - research_blog       │
└────────────────────────────────┘    │                          │
                                      │  oauth.scopes:           │
                                      │    - user                │
                                      │    - admin               │
                                      └──────────────────────────┘
```

---

## Referencing MCP Servers in Agents

### 1. Add to mcpServers Array

```yaml
metadata:
  mcpServers:
    - "content-manager"
    - "systemprompt-admin"
```

Server IDs must match keys in `mcp/*.yml` configuration files.

### 2. Document in systemPrompt

```yaml
metadata:
  systemPrompt: |
    # Content Agent

    ## Available MCP Tools

    You have access to the `content-manager` MCP server with these tools:

    ### create_blog_post
    Create a new blog post with title, content, and metadata.
    - Input: `title`, `content`, `category`, `tags`

    ### research_blog
    Research a topic before writing.
    - Input: `query`, `depth`

  mcpServers:
    - "content-manager"
```

### 3. Ensure OAuth Alignment

Agent security scopes MUST include MCP server required scopes:

```yaml
# Agent
card:
  security:
    - oauth2: ["user", "admin"]  # Must include MCP server scopes

# MCP Server
oauth:
  required: true
  scopes: ["user", "admin"]      # Agent must have these
```

---

## Using Skills with MCP Servers

Skills provide **instructions** for how to use MCP server **tools**.

### Skill File Example

`services/skills/linkedin_post_writing/SKILL.md`:

```markdown
# LinkedIn Post Writing Skill

## Process

1. **Research Phase**
   Use the `research_blog` tool to gather background information
   on the topic before writing.

2. **Draft Generation**
   Use the `generate_social_content` tool with platform="linkedin"
   to create the initial draft.

3. **Refinement**
   Review and adjust based on these guidelines:
   - Hook in first line
   - 1300 characters max
   - Professional tone

## Tool Usage

When creating LinkedIn content:
```
generate_social_content({
  platform: "linkedin",
  topic: "[user topic]",
  tone: "professional",
  include_hashtags: true
})
```
```

### Agent Orchestration

When a user invokes a skill:

1. Agent loads skill instructions from `SKILL.md`
2. Agent follows skill process steps
3. Agent calls MCP tools as directed by skill
4. Agent formats output per skill guidelines

---

## Model Configuration Precedence

AI model settings cascade with this precedence:

```
Tool Config (highest) > Agent Config > Global Config (lowest)
```

### Global Default

`services/ai/config.yml`:

```yaml
providers:
  default: "gemini"

  gemini:
    default_model: "gemini-2.5-flash"
    models:
      gemini-2.5-flash:
        max_output_tokens: 8192
```

### Agent Override

```yaml
metadata:
  provider: "gemini"
  model: "gemini-3-pro-preview"    # Overrides global
```

### Tool Override

```yaml
tools:
  create_blog_post:
    model_config:
      provider: gemini
      model: gemini-3-pro-preview  # Overrides agent and global
      max_output_tokens: 32768
      thinking_level: low
```

---

## Port Allocation

### Reserved Ranges

| Component | Port Range | Example |
|-----------|------------|---------|
| API Server | 8080 | Main API |
| MCP Servers | 5000-5999 | content-manager: 5003 |
| Agents | 9000-9999 | content: 9001 |

### Configuration

```yaml
settings:
  agentPortRange: [9000, 9999]
  mcpPortRange: [5000, 5999]
```

---

## Complete Example

### Content Agent System

**1. Agent** (`services/agents/content.yml`):

```yaml
agents:
  content:
    name: "content"
    port: 9001
    enabled: true

    card:
      skills:
        - id: "linkedin_post_writing"
          name: "LinkedIn Post Writing"
        - id: "research_blog"
          name: "Research Blog"

    metadata:
      systemPrompt: |
        # Content Marketing Agent

        ## Tools
        - `research_blog` - Research topics
        - `create_blog_post` - Generate posts
        - `generate_social_content` - Social posts

      mcpServers:
        - "content-manager"
```

**2. Skill** (`services/skills/linkedin_post_writing/config.yml`):

```yaml
id: linkedin_post_writing
name: "LinkedIn Post Writing"
description: "Create professional LinkedIn posts"
enabled: true
file: "SKILL.md"
assigned_agents:
  - content
tags:
  - linkedin
  - social_media
```

**3. MCP Server** (`services/mcp/content-manager.yml`):

```yaml
mcp_servers:
  content-manager:
    binary: "content-manager"
    port: 5003
    enabled: true

    oauth:
      required: true
      scopes: ["user", "admin"]

    tools:
      create_blog_post: {}
      generate_social_content: {}
      research_blog: {}
```

### Execution Flow

```
User: "Create a LinkedIn post about AI cost savings"
         │
         ▼
┌────────────────────────────┐
│ Agent "content" loaded     │
│                            │
│ Skills: [linkedin_post_writing, research_blog]
│ MCP: [content-manager]     │
└────────────┬───────────────┘
             │
             ▼
┌────────────────────────────┐
│ Agent loads skill:         │
│ linkedin_post_writing      │
│                            │
│ Reads SKILL.md instructions│
└────────────┬───────────────┘
             │
             ▼
┌────────────────────────────┐
│ Agent calls MCP tool:      │
│ research_blog              │
│                            │
│ Input: {query: "AI cost"}  │
└────────────┬───────────────┘
             │
             ▼
┌────────────────────────────┐
│ Agent calls MCP tool:      │
│ generate_social_content    │
│                            │
│ Input: {platform: "linkedin", topic: "..."}
└────────────┬───────────────┘
             │
             ▼
┌────────────────────────────┐
│ Agent formats per skill:   │
│ - Hook in first line       │
│ - 1300 chars max           │
│ - Professional tone        │
└────────────┬───────────────┘
             │
             ▼
         Response
```

---

## See Also

- [../architecture/overview.md](../architecture/overview.md) - MCP extension architecture
- [../architecture/boundaries.md](../architecture/boundaries.md) - Module boundaries
- [../implementation/prompts.md](../implementation/prompts.md) - Prompt implementation
- [../implementation/tools.md](../implementation/tools.md) - Tool implementation

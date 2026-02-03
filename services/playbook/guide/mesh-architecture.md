---
title: "Multi-Agent Mesh Architecture Guide"
description: "Architecture documentation for the multi-agent blog mesh system. Understand how agents coordinate, communicate, and share context."
keywords:
  - architecture
  - mesh
  - multi-agent
  - a2a
  - orchestration
category: guide
---

# Multi-Agent Mesh Architecture Guide

This guide explains the architecture of the multi-agent blog mesh system. It covers how agents coordinate, communicate, and share context to create blog content.

---

## Overview

The blog mesh is a coordinated system of specialised agents:

```
┌─────────────────────────────────────────────────────────────────┐
│                    SYSTEMPROMPT HUB                              │
│                    (Central - Port 9020)                         │
│  - Discord notifications                                         │
│  - Memory management                                             │
│  - Cross-agent communications                                    │
│  MCP: systemprompt, soul                                         │
└──────────────────────────────┬──────────────────────────────────┘
                               │
              summaries        │        updates
                               │
┌──────────────────────────────┴──────────────────────────────────┐
│                    BLOG ORCHESTRATOR                             │
│                    (Superagent - Port 9030)                      │
│  - Reads content_orchestration playbook                          │
│  - Routes to blog agents                                         │
│  - Coordinates workflow                                          │
│  MCP: systemprompt                                               │
└──────────────────────────────┬──────────────────────────────────┘
                               │
          delegates via        │
          A2A protocol         │
                               │
         ┌─────────────────────┴─────────────────────┐
         │                                           │
         ▼                                           ▼
┌─────────────────────┐                   ┌─────────────────────┐
│   BLOG TECHNICAL    │                   │   BLOG NARRATIVE    │
│   Port 9040         │                   │   Port 9050         │
│                     │                   │                     │
│   Skills:           │                   │   Skills:           │
│   - edwards_voice   │                   │   - edwards_voice   │
│   - technical_      │                   │   - blog_writing    │
│     content_writing │                   │   - research_blog   │
│   - research_blog   │                   │   - blog_image_     │
│   - blog_image_     │                   │     generation      │
│     generation      │                   │                     │
│                     │                   │   MCP: soul         │
│   MCP: soul         │                   │                     │
│                     │                   │   Content:          │
│   Content:          │                   │   - Personal        │
│   - Contrarian      │                   │     stories         │
│     deep-dives      │                   │   - Lessons         │
│   - Architecture    │                   │     learned         │
│   - Tech analysis   │                   │   - Tutorials       │
└─────────────────────┘                   └─────────────────────┘
         │                                           │
         └─────────────────────┬─────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SHARED INFRASTRUCTURE                         │
│  - soul MCP (memory + blog tools)                                │
│  - systemprompt MCP (CLI access)                                 │
│  - Skills (loaded into agent context)                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Agent Roles

### SystemPrompt Hub (Port 9020)

**Purpose:** Central nervous system for the mesh.

**Responsibilities:**
- Receive workflow status updates
- Send Discord notifications
- Store important decisions in memory
- Coordinate cross-agent communications

**MCP Servers:**
- `systemprompt` - CLI access for Discord, playbooks
- `soul` - Memory storage and retrieval

**Message Types:**
- `WORKFLOW_START: <description>` - Blog creation started
- `WORKFLOW_COMPLETE: slug=<slug>` - Blog published
- `WORKFLOW_FAILED: <reason>` - Something went wrong
- `STATUS_UPDATE: <message>` - Progress update

### Blog Orchestrator (Port 9030)

**Purpose:** Workflow coordinator and router.

**Responsibilities:**
- Analyse incoming requests
- Route to appropriate blog agent
- Create shared contexts
- Report status to hub

**MCP Servers:**
- `systemprompt` - CLI access for agent messaging, contexts

**Routing Rules:**

| Content Type | Route To | Keywords |
|--------------|----------|----------|
| Technical deep-dive | `blog_technical` | architecture, contrarian, analysis, why X is wrong |
| Personal narrative | `blog_narrative` | story, lessons, tutorial, how I, personal |

### Blog Technical (Port 9040)

**Purpose:** Contrarian technical deep-dives.

**Responsibilities:**
- Research topics using soul MCP
- Write 4000-6000 word technical posts
- Challenge conventional wisdom with evidence
- Generate featured images

**Skills:**
- `edwards_voice` - Voice and tone
- `technical_content_writing` - Contrarian structure
- `research_blog` - Topic research
- `blog_image_generation` - Featured images

**MCP Servers:**
- `soul` - Blog tools, memory

**Content Structure:**
```
# [Title]
## Prelude
## The Orthodoxy
## The Cracks
## The Deeper Truth
## Implications
## Conclusion
```

### Blog Narrative (Port 9050)

**Purpose:** Personal narratives and tutorials.

**Responsibilities:**
- Research topics using soul MCP
- Write 3500-5000 word narrative posts
- Balance 60% story, 40% technical
- Generate featured images

**Skills:**
- `edwards_voice` - Voice and tone
- `blog_writing` - Narrative structure
- `research_blog` - Topic research
- `blog_image_generation` - Featured images

**MCP Servers:**
- `soul` - Blog tools, memory

**Content Structure:**
```
# [Title]
## Prelude
## The Problem
## The Journey
## The Lesson
## Conclusion
```

---

## Communication Patterns

### A2A Protocol

Agents communicate via Agent-to-Agent (A2A) protocol using CLI commands:

```bash
# Blocking (wait for response)
admin agents message <agent> -m "<message>" --blocking

# With timeout (for long operations)
admin agents message <agent> -m "<message>" --blocking --timeout 300

# With shared context
admin agents message <agent> -m "<message>" --context-id <id> --blocking
```

### Workflow Example

```
1. User -> Orchestrator
   admin agents message blog_orchestrator -m "Create technical blog about MCP" --blocking

2. Orchestrator -> Hub (notify start)
   admin agents message systemprompt_hub -m "WORKFLOW_START: technical blog on MCP"

3. Orchestrator -> Blog Agent (create)
   core contexts new --name "blog-mcp-architecture"
   admin agents message blog_technical -m "Create blog: MCP deep-dive..." --blocking --timeout 300

4. Blog Agent -> Soul MCP (tools)
   - research_blog -> artifact_id
   - create_blog_post -> content_id, slug

5. Orchestrator -> Hub (notify complete)
   admin agents message systemprompt_hub -m "WORKFLOW_COMPLETE: slug=mcp-architecture"

6. Hub -> Discord + Memory
   - plugins run discord send "Blog published: MCP Architecture"
   - memory_store: fact about blog creation
```

---

## MCP Server Integration

### systemprompt MCP

Provides CLI access to the SystemPrompt platform:

**Used by:** Hub, Orchestrator

**Key Commands:**
- `admin agents message <name> -m "<msg>" --blocking` - A2A messaging
- `core contexts new --name "<name>"` - Create shared context
- `core playbooks show <id>` - Read playbooks
- `plugins run discord send "<msg>"` - Discord notifications

### soul MCP

Provides memory and blog creation tools:

**Used by:** Hub, Blog Technical, Blog Narrative

**Memory Tools:**
- `memory_get_context` - Retrieve formatted memory
- `memory_store` - Store new memory
- `memory_search` - Search memories
- `memory_forget` - Soft-delete memory

**Blog Tools:**
- `research_blog` - Research a topic
- `create_blog_post` - Create and publish post
- `update_blog_post` - Update existing post

---

## Context Management

### Shared Contexts

Each workflow creates a named context for tracking:

```bash
core contexts new --name "blog-<topic-slug>"
```

### Context Passing

Pass context ID when routing to maintain workflow state:

```bash
admin agents message blog_technical -m "..." --context-id "blog-mcp-architecture" --blocking
```

### Context Cleanup

Contexts are not automatically cleaned up. Manage manually:

```bash
core contexts list
core contexts delete <context-id>
```

---

## Skills Architecture

Skills are loaded into agent context at startup:

### Skill Loading

Agent YAML references skills by ID:

```yaml
skills:
  - edwards_voice
  - technical_content_writing
  - research_blog
  - blog_image_generation
```

### Skill Content

Skills provide:
- Voice and tone guidelines
- Content structure templates
- Formatting rules
- Don'ts and anti-patterns

### Skill Sync

Skills must be synced to database:

```bash
systemprompt core skills sync --direction to-db -y
```

---

## Port Allocation

| Port | Agent | Purpose |
|------|-------|---------|
| 9000 | welcome | Default assistant |
| 9010 | soul | Memory agent |
| 9020 | systemprompt_hub | Central hub |
| 9030 | blog_orchestrator | Workflow coordinator |
| 9040 | blog_technical | Technical content |
| 9050 | blog_narrative | Narrative content |

Reserved ranges:
- 9000-9019: Core agents
- 9020-9029: Hub and orchestrators
- 9030-9039: Orchestrators
- 9040-9099: Specialised workers

---

## Error Handling

### Timeout Handling

For long-running blog creation:

```bash
# Use longer timeout
admin agents message blog_technical -m "..." --blocking --timeout 600

# Or poll for completion
admin agents message blog_technical -m "..."  # Returns task-id
admin agents task blog_technical --task-id <id>  # Poll status
```

### Failure Recovery

Orchestrator handles failures:

1. Logs the error
2. Notifies hub: `WORKFLOW_FAILED: <reason>`
3. Does not auto-retry (requires user intervention)

### Hub Memory

Hub stores failures in memory for analysis:

```bash
admin agents message systemprompt_hub -m "What workflows failed recently?" --blocking
```

---

## Monitoring

### Agent Status

```bash
admin agents registry           # Running agents
admin agents status <name>      # Individual status
admin agents logs <name> -n 50  # Recent logs
```

### Workflow Tracking

```bash
# Check hub for recent activity
admin agents message systemprompt_hub -m "Recent workflow summary" --blocking

# Check pending tasks
admin agents task blog_orchestrator --task-id <id>
```

### Health Checks

```bash
admin agents validate           # Validate all configs
admin agents tools <name>       # List available tools
```

---

## Extending the Mesh

### Adding a New Blog Agent

1. Create agent YAML in `services/agents/`:
   - Choose unique port (9060+)
   - Reference appropriate MCP servers
   - List required skills
   - Write system prompt

2. Update orchestrator routing rules

3. Add to mesh playbook

4. Test communication:
   ```bash
   admin agents message <new_agent> -m "test" --blocking
   ```

### Adding New Skills

1. Create skill in `services/skills/<name>/`:
   - `config.yaml` - Metadata
   - `index.md` - Content

2. Sync to database:
   ```bash
   systemprompt core skills sync --direction to-db -y
   ```

3. Reference in agent YAML

---

## Best Practices

### Agent Design

- **Single responsibility** - Each agent does one thing well
- **Clear routing** - Unambiguous criteria for content type
- **Skill separation** - Reusable skills across agents
- **MCP isolation** - Only grant MCP access needed

### Communication

- **Always notify hub** - Start and end of workflows
- **Use timeouts** - Prevent hanging operations
- **Create contexts** - Track workflow state
- **Log failures** - Store in hub memory

### Content Quality

- **Research first** - Always use research_blog before writing
- **Follow skills** - Respect voice and structure guidelines
- **British English** - Consistent language throughout
- **Cite sources** - Inline markdown links from research

---

## Related Playbooks

- [Content Orchestration](../content/orchestration.md) - Using the orchestrator
- [Mesh Management](../cli/mesh.md) - Managing mesh agents
- [Agents CLI](agents.md) - Agent commands
- [Blog Content](../content/blog.md) - Direct blog creation

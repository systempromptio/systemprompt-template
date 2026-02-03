---
title: "Agent Mesh Management Playbook"
description: "Manage multi-agent mesh systems. Start, stop, monitor, and troubleshoot agent mesh deployments."
keywords:
  - mesh
  - agents
  - multi-agent
  - orchestration
  - management
category: cli
---

# Agent Mesh Management Playbook

Manage multi-agent mesh systems. This playbook covers starting, stopping, monitoring, and troubleshooting mesh deployments.

> **Help**: `admin agents --help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Mesh Overview

A mesh is a coordinated group of agents working together. The blog mesh consists of:

| Agent | Port | Role |
|-------|------|------|
| `systemprompt_hub` | 9020 | Central communications, Discord, memory |
| `blog_orchestrator` | 9030 | Workflow coordination, routing |
| `blog_technical` | 9040 | Technical deep-dive content |
| `blog_narrative` | 9050 | Personal narrative content |

---

## List Mesh Agents

```json
{ "command": "admin agents list --enabled" }
```

Filter by port range (mesh agents use 9020-9050):

```json
{ "command": "admin agents list" }
```

---

## Check Mesh Status

### All Agents

```json
{ "command": "admin agents registry" }
```

### Individual Agent

```json
{ "command": "admin agents status systemprompt_hub" }
{ "command": "admin agents status blog_orchestrator" }
{ "command": "admin agents status blog_technical" }
{ "command": "admin agents status blog_narrative" }
```

---

## View Agent Configuration

```json
{ "command": "admin agents show systemprompt_hub" }
{ "command": "admin agents show blog_orchestrator" }
{ "command": "admin agents show blog_technical" }
{ "command": "admin agents show blog_narrative" }
```

---

## View Agent Logs

```json
{ "command": "admin agents logs systemprompt_hub -n 50" }
{ "command": "admin agents logs blog_orchestrator -n 50" }
{ "command": "admin agents logs blog_technical -n 50" }
{ "command": "admin agents logs blog_narrative -n 50" }
```

---

## Enable/Disable Mesh Agents

### Enable All Mesh Agents

```json
{ "command": "admin agents edit systemprompt_hub --enable" }
{ "command": "admin agents edit blog_orchestrator --enable" }
{ "command": "admin agents edit blog_technical --enable" }
{ "command": "admin agents edit blog_narrative --enable" }
```

### Disable All Mesh Agents

```json
{ "command": "admin agents edit systemprompt_hub --disable" }
{ "command": "admin agents edit blog_orchestrator --disable" }
{ "command": "admin agents edit blog_technical --disable" }
{ "command": "admin agents edit blog_narrative --disable" }
```

---

## Restart Mesh Agents

### Restart All Agents

```json
{ "command": "infra services restart --agents" }
```

### Restart Individual Agent

```json
{ "command": "infra services restart agent systemprompt_hub" }
{ "command": "infra services restart agent blog_orchestrator" }
{ "command": "infra services restart agent blog_technical" }
{ "command": "infra services restart agent blog_narrative" }
```

---

## Validate Agent Configurations

```json
{ "command": "admin agents validate" }
```

```json
{ "command": "admin agents validate systemprompt_hub" }
{ "command": "admin agents validate blog_orchestrator" }
{ "command": "admin agents validate blog_technical" }
{ "command": "admin agents validate blog_narrative" }
```

---

## Test Mesh Communication

### Test Hub

```json
{ "command": "admin agents message systemprompt_hub -m \"TEST: Verify mesh hub is responding\" --blocking" }
```

### Test Orchestrator

```json
{ "command": "admin agents message blog_orchestrator -m \"What agents can you route to?\" --blocking" }
```

### Test Blog Agents

```json
{ "command": "admin agents message blog_technical -m \"What skills do you have?\" --blocking" }
{ "command": "admin agents message blog_narrative -m \"What skills do you have?\" --blocking" }
```

---

## List Agent Tools

Check what MCP tools each agent has access to:

```json
{ "command": "admin agents tools systemprompt_hub" }
{ "command": "admin agents tools blog_orchestrator" }
{ "command": "admin agents tools blog_technical" }
{ "command": "admin agents tools blog_narrative" }
```

---

## Monitor Mesh Workflows

### Check Recent Activity

```json
{ "command": "admin agents message systemprompt_hub -m \"What workflows have run recently?\" --blocking" }
```

### Check Pending Tasks

```json
{ "command": "admin agents task blog_orchestrator --task-id [task-id]" }
```

---

## A2A Communication

Agents communicate via A2A (Agent-to-Agent) protocol:

### Orchestrator to Hub

```
admin agents message systemprompt_hub -m "WORKFLOW_START: technical blog on X" --blocking
```

### Orchestrator to Blog Agent

```
admin agents message blog_technical -m "Create blog post about X" --blocking --timeout 300
```

### With Context Sharing

```
admin agents message blog_technical -m "Create blog..." --context-id "blog-xyz" --blocking --timeout 300
```

---

## Mesh Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    SYSTEMPROMPT HUB (9020)                      │
│  - Discord notifications                                        │
│  - Memory management                                            │
│  - Cross-agent coordination                                     │
│  MCP: systemprompt, soul                                        │
└──────────────────────────────┬──────────────────────────────────┘
                               │
              summaries        │        updates
                               │
┌──────────────────────────────┴──────────────────────────────────┐
│                   BLOG ORCHESTRATOR (9030)                       │
│  - Reads content_orchestration playbook                          │
│  - Routes to specialised agents                                  │
│  - Coordinates workflow                                          │
│  MCP: systemprompt                                               │
└──────────────────────────────┬──────────────────────────────────┘
                               │
          A2A routing          │
                               │
         ┌─────────────────────┴─────────────────────┐
         │                                           │
         ▼                                           ▼
┌─────────────────────┐                   ┌─────────────────────┐
│ BLOG TECHNICAL      │                   │ BLOG NARRATIVE      │
│ (9040)              │                   │ (9050)              │
│                     │                   │                     │
│ Skills:             │                   │ Skills:             │
│ - edwards_voice     │                   │ - edwards_voice     │
│ - technical_content │                   │ - blog_writing      │
│ - research_blog     │                   │ - research_blog     │
│ - blog_image_gen    │                   │ - blog_image_gen    │
│                     │                   │                     │
│ MCP: soul           │                   │ MCP: soul           │
│                     │                   │                     │
│ Content:            │                   │ Content:            │
│ - Contrarian takes  │                   │ - Personal stories  │
│ - Architecture      │                   │ - Lessons learned   │
│ - Tech analysis     │                   │ - Tutorials         │
└─────────────────────┘                   └─────────────────────┘
```

---

## Troubleshooting

### Agent Not Responding

1. Check status: `admin agents status [name]`
2. Check logs: `admin agents logs [name] -n 100`
3. Restart agent: `infra services restart agent [name]`

### Agent Not in Registry

1. Verify enabled: `admin agents show [name]`
2. Enable if needed: `admin agents edit [name] --enable`
3. Restart: `infra services restart --agents`

### Communication Timeout

1. Increase timeout: `--blocking --timeout 300`
2. Check agent load: `admin agents status [name]`
3. Consider async: Remove `--blocking`, use task polling

### Validation Errors

1. Run validation: `admin agents validate [name]`
2. Check MCP server refs: Ensure MCP servers exist
3. Check skill refs: Ensure skills are synced

### Discord Not Working

1. Verify config: `admin config show discord`
2. Test directly: `plugins run discord send "test"`
3. Check hub logs: `admin agents logs systemprompt_hub -n 100`

---

## Quick Reference

| Task | Command |
|------|---------|
| List mesh agents | `admin agents list --enabled` |
| Check registry | `admin agents registry` |
| Agent status | `admin agents status [name]` |
| Agent logs | `admin agents logs [name] -n 50` |
| Agent config | `admin agents show [name]` |
| Enable agent | `admin agents edit [name] --enable` |
| Disable agent | `admin agents edit [name] --disable` |
| Restart all | `infra services restart --agents` |
| Restart one | `infra services restart agent [name]` |
| Validate | `admin agents validate` |
| Test message | `admin agents message [name] -m "test" --blocking` |
| List tools | `admin agents tools [name]` |

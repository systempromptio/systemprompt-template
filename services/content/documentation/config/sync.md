---
title: "Code Sync"
description: "Synchronize configuration between local development and SystemPrompt Cloud. Push, pull, and resolve conflicts."
author: "SystemPrompt Team"
slug: "config/sync"
keywords: "sync, synchronization, push, pull, configuration"
image: "/files/images/docs/config-sync.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Code Sync

Synchronize configuration between your local environment and SystemPrompt Cloud. Keep skills, agents, content, and MCP servers in sync across environments.

## Sync Overview

```
┌─────────────────────┐                    ┌─────────────────────┐
│       LOCAL         │                    │        CLOUD        │
│                     │      push          │                     │
│  services/          │  ─────────────▶    │  Cloud Database     │
│  ├── agents/        │                    │  ├── agents         │
│  ├── skills/        │      pull          │  ├── skills         │
│  ├── mcp/           │  ◀─────────────    │  ├── mcp_servers    │
│  └── content/       │                    │  └── content        │
└─────────────────────┘                    └─────────────────────┘
```

## Push Local to Cloud

Push your local configuration to the cloud tenant:

```bash
# Push all configuration
systemprompt cloud sync push --all

# Push specific types
systemprompt cloud sync push agents
systemprompt cloud sync push skills
systemprompt cloud sync push mcp
systemprompt cloud sync push content
```

## Pull Cloud to Local

Pull cloud configuration to your local files:

```bash
# Pull all configuration
systemprompt cloud sync pull --all

# Pull specific types
systemprompt cloud sync pull agents
systemprompt cloud sync pull skills
```

## Local Sync (Files to Database)

Sync local files to the local database (not cloud):

```bash
# Sync to local database
systemprompt cloud sync local --all --direction to-db -y

# Sync specific types
systemprompt cloud sync local agents --direction to-db -y
systemprompt cloud sync local skills --direction to-db -y
systemprompt cloud sync local content --direction to-db -y
```

## What Gets Synced

| Type | Local Path | Description |
|------|------------|-------------|
| Agents | `services/agents/*.yaml` | Agent definitions and A2A cards |
| Skills | `services/skills/*.yaml` | Skill definitions |
| MCP Servers | `services/mcp/*.yaml` | MCP server configurations |
| Content | `services/content/**/*.md` | Blog posts, documentation |
| Playbooks | `services/playbook/**/*.md` | Machine instruction sets |

## Conflict Resolution

When local and cloud versions differ:

```bash
# Preview changes before sync
systemprompt cloud sync push --dry-run

# Force local to override cloud
systemprompt cloud sync push --force

# Force cloud to override local
systemprompt cloud sync pull --force
```

### Conflict Strategies

| Strategy | Flag | Behavior |
|----------|------|----------|
| Preview | `--dry-run` | Show diff without changes |
| Local wins | `push --force` | Overwrite cloud with local |
| Cloud wins | `pull --force` | Overwrite local with cloud |
| Merge | Default | Fail on conflict, require resolution |

## Dry Run Mode

Preview what would be synced without making changes:

```bash
# See what would be pushed
systemprompt cloud sync push --all --dry-run

# See what would be pulled
systemprompt cloud sync pull --all --dry-run
```

## Sync Specific Files

```bash
# Sync a single agent
systemprompt cloud sync push agents/welcome.yaml

# Sync a single skill
systemprompt cloud sync push skills/blog-writer.yaml
```

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Auth error | Session expired | Run `systemprompt cloud auth login` |
| Conflict | Local and cloud differ | Use `--dry-run` to compare, then `--force` |
| Missing files | File not tracked | Check file exists in correct directory |
| Schema error | Invalid YAML | Validate with `systemprompt admin config validate` |

## Quick Reference

| Task | Command |
|------|---------|
| Push all | `systemprompt cloud sync push --all` |
| Pull all | `systemprompt cloud sync pull --all` |
| Push agents | `systemprompt cloud sync push agents` |
| Preview push | `systemprompt cloud sync push --all --dry-run` |
| Force push | `systemprompt cloud sync push --all --force` |
| Local sync | `systemprompt cloud sync local --all --direction to-db -y` |
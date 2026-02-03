---
title: "Contexts Management Playbook"
description: "Manage conversation contexts for agent interactions."
author: "SystemPrompt"
slug: "cli-contexts"
keywords: "contexts, conversations, agents, state"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Contexts Management Playbook

Manage conversation contexts for agent interactions.

---

## List Contexts

```json
{ "command": "core contexts list" }
```

Shows all contexts with ID, name, task count, message count, last updated, and active marker (*).

---

## Show Context Details

Short IDs (first 8 chars) work for most commands.

```json
{ "command": "core contexts show <id>" }
```

---

## Create New Context

```json
{ "command": "core contexts create --name \"my-context\"" }
{ "command": "core contexts new --name \"my-context\"" }
```

Use `new` to create and set as active in one step (recommended).

---

## Switch Active Context

```json
{ "command": "core contexts use <id>" }
```

---

## Rename Context

```json
{ "command": "core contexts edit <id> --name \"renamed-context\"" }
```

---

## Delete Context

```json
{ "command": "core contexts delete <id> -y" }
```

---

## Context Workflows

### Fresh Conversation with Agent

```json
{ "command": "core contexts new --name \"my-research-task\"" }
{ "command": "admin agents message <name> -m \"Research topic\" --blocking" }
```

### Multi-Agent Shared Context

All agents share the active context and can reference each other's work:

```json
{ "command": "core contexts new --name \"content-pipeline\"" }
{ "command": "admin agents message researcher -m \"Research AI costs\" --blocking" }
{ "command": "admin agents message writer -m \"Create post from research\" --blocking" }
```

### Resume Previous Conversation

```json
{ "command": "core contexts list" }
{ "command": "core contexts use <context-id>" }
{ "command": "admin agents message <name> -m \"Continue from where we left off\" --blocking" }
```

---

## Troubleshooting

**Wrong context active** -- List with `core contexts list`, switch with `core contexts use <id>`.

**Context not found** -- Use full UUID if short ID is ambiguous.

**Too many contexts** -- Clean up old contexts with `core contexts delete <id> -y`.

---

## Quick Reference

| Task | Command |
|------|---------|
| List contexts | `core contexts list` |
| Show context | `core contexts show <id>` |
| Create context | `core contexts create --name "x"` |
| Create and activate | `core contexts new --name "x"` |
| Switch context | `core contexts use <id>` |
| Rename context | `core contexts edit <id> --name "x"` |
| Delete context | `core contexts delete <id> -y` |
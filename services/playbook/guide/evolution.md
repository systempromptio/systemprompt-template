---
title: "Evolution of AI Agents"
description: "Interactive guide through the 4 levels of AI agent evolution. From chat to mesh orchestration."
author: "SystemPrompt"
slug: "guide-evolution"
keywords: "evolution, levels, chat, tools, agents, mesh, orchestration, tutorial"
image: ""
kind: "playbook"
public: true
tags: ["evolution", "tutorial", "getting-started"]
published_at: "2026-02-03"
updated_at: "2026-02-03"
---

# Evolution of AI Agents

Welcome to the endgame. This guide walks you through the 4 levels of AI agent evolution.

---

## Prerequisites

Verify your session is active:

```json
{ "command": "admin session show" }
```

Verify agents are running:

```json
{ "command": "admin agents list --enabled" }
```

---

## Level 1: Chat (The Copy-Paste Era)

**You ask. AI answers. You copy and paste.**

Every interaction is isolated. The human is the middleware.

### Step 1.1: Talk to the Welcome Agent

```json
{ "command": "admin agents message welcome -m 'Hello!' --blocking" }
```

### Step 1.2: View the Request Trace

```json
{ "command": "infra logs request list --limit 1" }
```

**Characteristics:**
- Single-turn conversations
- Manual data transfer
- No persistent context

---

## Level 2: Tools (The Function-Calling Era)

**AI invokes tools. It reads data, calls APIs, writes files.**

Structured outputs, but you still orchestrate every step.

### Step 2.1: List Available MCP Tools

```json
{ "command": "plugins mcp tools" }
```

### Step 2.2: Call an MCP Tool Directly

```json
{ "command": "plugins mcp call content-manager create_blog_post --args '{\"skill_id\": \"announcement_writing\", \"slug\": \"test-announcement\", \"description\": \"Test post\", \"keywords\": [\"test\"], \"instructions\": \"Write a brief test announcement.\"}'" }
```

**Characteristics:**
- AI invokes tools
- Structured outputs
- Human as orchestrator

---

## Level 3: Agents (The Autonomous Era)

**Agents run multi-step workflows. Define the goal - they find the path.**

Agents reason, plan, use tools, and iterate autonomously.

### Step 3.1: Ask Agent to Create Blog Content

The welcome agent has MCP tools and will use them autonomously:

```json
{ "command": "admin agents message welcome -m 'Create a brief announcement blog post about systemprompt.io' --blocking --timeout 120" }
```

### Step 3.2: Follow the Discourse

The agent will research and ask for confirmation before creating. Respond to continue:

```json
{ "command": "admin agents message welcome -m 'Yes, proceed with creating the blog post' --blocking --timeout 120" }
```

### Step 3.3: View Created Content

```json
{ "command": "core content list --source blog --limit 5" }
```

**Characteristics:**
- Goal-driven autonomy
- Multi-step reasoning
- Human as supervisor

---

## Level 4: Mesh (The Orchestration Era)

**Superagents coordinate specialized agents. Ask once - the mesh delivers.**

Load a playbook in Claude Code and follow the discourse.

### Step 4.1: Load the Orchestration Playbook

```json
{ "command": "core playbooks show content_orchestration" }
```

### Step 4.2: Use the Blog Orchestrator

The orchestrator routes to specialized agents based on content type:

```json
{ "command": "admin agents message blog_orchestrator -m 'Create a NARRATIVE blog post about the evolution of AI agents.\n\nStory: The journey from simple chatbots to coordinated agent meshes.\nLesson: Specialization and orchestration beat monolithic AI systems.\nReader outcome: Teams will understand the maturity model for AI agent adoption.' --blocking --timeout 600" }
```

### Step 4.3: Check Workflow Status

```json
{ "command": "admin agents message systemprompt_hub -m 'What is the latest workflow status?' --blocking" }
```

### Step 4.4: Publish Content

```json
{ "command": "infra jobs run publish_pipeline" }
```

**Characteristics:**
- Multi-agent coordination
- Hierarchical delegation
- Human as director

---

## Quick Reference

| Level | Command |
|-------|---------|
| Level 1: Chat | `admin agents message welcome -m 'Hello!' --blocking` |
| Level 2: Tools | `plugins mcp tools` |
| Level 3: Agents | `admin agents message welcome -m 'Create content' --blocking` |
| Level 4: Mesh | `core playbooks show content_orchestration` |

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Agent not responding | `admin agents status welcome` |
| No MCP tools listed | `plugins mcp status` |
| Content not created | `core content search "keyword"` |
| Orchestrator timeout | Increase timeout: `--timeout 600` |

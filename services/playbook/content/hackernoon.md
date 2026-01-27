---
title: "HackerNoon Content Creation Playbook"
description: "Create deep technical dives with contrarian attitude and annotated code for skeptical senior developers. Production reality over polished demos."
keywords:
  - hackernoon
  - technical
  - code
  - deep-dive
---

# HackerNoon Content Creation Playbook

Create deep technical dives with contrarian attitude and annotated code for skeptical senior developers. Production reality over polished demos.

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.** Verify your session and profile before starting.

**IMPORTANT: NEVER start, stop, or restart services. The API is already running.**

---

## CRITICAL: Context Management

**New content = New context. ALWAYS.** See the [Contexts Playbook](../cli/contexts.md) for full context management commands.

**WRONG:** Reusing `hackernoon-general` context for multiple articles
**RIGHT:** `hackernoon-agent-mesh-critique`, `hackernoon-rag-postmortem`, `hackernoon-llm-benchmarks` (one context per article)

---

## CRITICAL: One Step = One Message

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Step 3 | Plan | YES |
| Step 4 | Research | YES |
| Step 5 | Create | YES |
| Step 6 | Image | YES |

---

## Step 1: Create a FRESH Context

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"hackernoon-[topic-slug]\"" }
```

## Step 2: Review Performance and Trends

```json
// MCP: systemprompt_cli
{ "command": "analytics content stats --since 30d" }
```

## Step 3: Plan the Technical Deep Dive

HackerNoon readers expect code, data, and contrarian takes.

## Step 4: Research

```json
// MCP: systemprompt_cli
{ "command": "admin agents message hackernoon -m \"Research this topic.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

## Step 5: Create Content

```json
// MCP: systemprompt_cli
{ "command": "admin agents message hackernoon -m \"Create the article with annotated code examples.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 300" }
```

## Step 6: Generate Image

```json
// MCP: systemprompt_cli
{ "command": "admin agents message hackernoon -m \"Generate a featured image.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 60" }
```

## Step 7: Publish and Verify

```json
// MCP: systemprompt_cli
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run publish_content" }
{ "command": "cloud sync local content --direction to-disk --source hackernoon -y" }
```

## Step 8: Update AI Provenance (MANDATORY AFTER CRUD)

**CRITICAL: After ANY content CRUD operation (Create, Read, Update, Delete), agents MUST update the AI Provenance metadata.**

AI Provenance is rendered in the content template as a transparency block showing readers how the content was created.

### 8.1 Update Provenance via CLI

```json
// MCP: systemprompt_cli - Set provenance fields
{ "command": "core content edit --slug [slug] --source hackernoon --set agent=hackernoon --set agent_summary=\"Created article: [Title] - [Why]\" --set category=\"Technical Deep Dive\"" }
```

Verify provenance is set:

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug [slug] --source hackernoon" }
```

### 8.2 Agent Summary Guidelines

The `agent_summary` is displayed as "Why This Was Created". It should:
- Be concise (1-2 sentences)
- Explain the PURPOSE, not just the action
- Include the content title
- Reference the contrarian angle if relevant

**GOOD:** `"Created article: Agent Mesh Architecture Critique - challenging single-agent patterns with production data."`
**BAD:** `"Created article"` (too vague)

**Provenance is NOT optional. All AI-generated content MUST have complete provenance metadata.**

---

## Content Types

| Type | Length | Use Case |
|------|--------|----------|
| `hackernoon_article` | 2,000-4,000 words | Deep technical dives |

## Format Requirements

- Annotated code blocks (minimum 3)
- Production metrics and data
- Contrarian take clearly stated
- British English

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Content not created | Verify with `core content search` |
| Agent says "created" but doesn't exist | **Create a NEW context** |

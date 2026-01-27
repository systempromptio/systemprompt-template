---
title: "Substack Technical Newsletter Playbook"
description: "Create deep technical newsletters explaining the tyingshoelaces agentic mesh architecture with Rust code walkthroughs and architecture diagrams."
keywords:
  - substack
  - newsletter
  - technical
  - rust
  - architecture
---

# Substack Technical Newsletter Playbook

Create deep technical newsletters explaining the tyingshoelaces agentic mesh architecture with Rust code walkthroughs and architecture diagrams. Build a technical audience by documenting production agentic systems.

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.** Verify your session and profile before starting.

**IMPORTANT: NEVER start, stop, or restart services. The API is already running.**

---

## CRITICAL: Context Management

**New content = New context. ALWAYS.** See the [Contexts Playbook](../cli/contexts.md) for full context management commands.

**WRONG:** Reusing `substack-general` context for multiple issues
**RIGHT:** `substack-extension-traits`, `substack-mcp-architecture`, `substack-job-scheduling` (one context per issue)

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
{ "command": "core contexts new --name \"substack-[topic-slug]\"" }
```

## Step 2: Review Performance

```json
// MCP: systemprompt_cli
{ "command": "analytics content stats --since 30d" }
```

## Step 3: Plan the Newsletter Issue

Focus on a specific component of the agentic mesh architecture.

## Step 4: Research

```json
// MCP: systemprompt_cli
{ "command": "admin agents message substack -m \"Research this architecture component.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

## Step 5: Create Content

```json
// MCP: systemprompt_cli
{ "command": "admin agents message substack -m \"Create the newsletter with Rust code walkthroughs.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 300" }
```

## Step 6: Generate Diagrams

```json
// MCP: systemprompt_cli
{ "command": "admin agents message substack -m \"Generate architecture diagrams.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 60" }
```

## Step 7: Publish and Verify

```json
// MCP: systemprompt_cli
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run publish_content" }
{ "command": "cloud sync local content --direction to-disk --source substack -y" }
```

## Step 8: Update AI Provenance (MANDATORY AFTER CRUD)

**CRITICAL: After ANY content CRUD operation (Create, Read, Update, Delete), agents MUST update the AI Provenance metadata.**

AI Provenance is rendered in the content template as a transparency block showing readers how the content was created.

### 8.1 Update Provenance via CLI

```json
// MCP: systemprompt_cli - Set provenance fields
{ "command": "core content edit --slug [slug] --source substack --set agent=substack --set agent_summary=\"Created newsletter: [Title] - [Why]\" --set category=\"Architecture Deep Dive\"" }
```

Verify provenance is set:

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug [slug] --source substack" }
```

### 8.2 Agent Summary Guidelines

The `agent_summary` is displayed as "Why This Was Created". It should:
- Be concise (1-2 sentences)
- Explain the PURPOSE, not just the action
- Include the content title
- Reference the architecture component if relevant

**GOOD:** `"Created newsletter: Extension Traits Deep Dive - explaining the trait system for building custom extensions."`
**BAD:** `"Created newsletter"` (too vague)

**Provenance is NOT optional. All AI-generated content MUST have complete provenance metadata.**

---

## Content Types

| Type | Length | Use Case |
|------|--------|----------|
| `substack_newsletter` | 2,000-3,500 words | Technical deep dives |

## Format Requirements

- Rust code examples with annotations
- Architecture diagrams
- Production context and lessons
- British English

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Content not created | Verify with `core content search` |
| Agent says "created" but doesn't exist | **Create a NEW context** |

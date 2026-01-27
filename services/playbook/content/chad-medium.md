---
title: "Chad Venture - Medium Content Creation Playbook"
description: "Create satirical Medium articles featuring Chad Venture, a fictional American tech bro whose journey from Stanford to unicorn lampoons startup culture."
keywords:
  - chad-venture
  - satire
  - medium
  - fiction
---

# Chad Venture - Medium Content Creation Playbook

Create satirical Medium articles featuring Chad Venture, a fictional American tech bro whose journey from Stanford to unicorn lampoons startup culture.

**CRITICAL: All content is FICTION. Every article MUST include the mandatory disclaimer footer.**

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.** Verify your session and profile before starting.

**IMPORTANT: NEVER start, stop, or restart services. The API is already running.**

---

## Character Reference: Chad Venture

**Full Name:** Chadwick Bainbridge Venture III
**Background:** Stanford "dropout" (changed majors), father is Sand Hill Road VC partner
**Current Venture:** CEO of Synergize.ai (4th startup)

**Voice Characteristics:**
- Uses "we" even when alone
- Drops VC names casually ("As Marc always says...")
- Describes failures as "learnings"

---

## CRITICAL: Context Management

**New content = New context. ALWAYS.**

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"chad-medium-[topic-slug]\"" }
```

---

## CRITICAL: One Step = One Message

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Plan | Define the satirical angle | YES |
| Research | Research the trend being mocked | YES |
| Create | Create the article | YES |
| Image | Generate featured image | YES |

---

## Step 1: Create Context

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"chad-medium-disruption-manifesto\"" }
```

## Step 2: Plan the Satire

Identify the startup culture phenomenon to satirise.

## Step 3: Research

```json
// MCP: systemprompt_cli
{ "command": "admin agents message chad_medium -m \"Research this startup trend.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

## Step 4: Create Content

```json
// MCP: systemprompt_cli
{ "command": "admin agents message chad_medium -m \"Create the satirical article.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 300" }
```

## Step 5: Generate Image

```json
// MCP: systemprompt_cli
{ "command": "admin agents message chad_medium -m \"Generate a featured image.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 60" }
```

## Step 6: Publish and Verify

```json
// MCP: systemprompt_cli
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run publish_content" }
{ "command": "cloud sync local content --direction to-disk --source chad-medium -y" }
```

## Content Types

| Type | Length | Use Case |
|------|--------|----------|
| `chad_medium_article` | 1,500-2,500 words | Satirical long-form |

## Format Requirements

- Personal narrative in Chad's voice
- Unearned confidence throughout
- **MANDATORY**: Fiction disclaimer footer

**Required disclaimer footer:**
```
---
*Chad Venture is a fictional character. This article is satirical commentary on startup culture. Any resemblance to actual tech bros is entirely coincidental (but probably accurate).*

*[tyingshoelaces.com/chad]*
```

---

## Step 7: Update AI Provenance (MANDATORY AFTER CRUD)

**CRITICAL: After ANY content CRUD operation (Create, Read, Update, Delete), agents MUST update the AI Provenance metadata.**

### 7.1 Update Provenance via CLI

```json
// MCP: systemprompt_cli - Set provenance fields
{ "command": "core content edit --slug [slug] --source chad-medium --set agent=chad_medium --set agent_summary=\"Created satirical article: [Title] - [Satire target]\" --set category=\"Satirical Fiction\"" }
```

Verify provenance is set:

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug [slug] --source chad-medium" }
```

### 7.2 Agent Summary Guidelines

The `agent_summary` is displayed as "Why This Was Created". It should:
- Be concise (1-2 sentences)
- Explain the PURPOSE (what startup culture phenomenon is being satirised)
- Include the content title
- Note this is FICTION

**GOOD:** `"Created satirical article: How I Disrupted Disruption - lampooning the 'disruption' obsession in tech culture."`
**BAD:** `"Created article"` (too vague)

**Provenance is NOT optional. All AI-generated content MUST have complete provenance metadata.**

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Missing disclaimer | Regenerate - disclaimer is MANDATORY |
| Content not created | Verify with `core content search` |

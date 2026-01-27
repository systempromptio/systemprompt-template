---
title: "Chad Venture - Twitter Content Creation Playbook"
description: "Create satirical Twitter threads featuring Chad Venture, a fictional American tech bro dispensing startup wisdom with unearned confidence."
keywords:
  - chad-venture
  - satire
  - twitter
  - fiction
---

# Chad Venture - Twitter Content Creation Playbook

Create satirical Twitter threads featuring Chad Venture, a fictional American tech bro dispensing startup "wisdom" with unearned confidence.

**CRITICAL: All content is FICTION. Every thread MUST include the disclaimer in the final tweet.**

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.** Verify your session and profile before starting.

**IMPORTANT: NEVER start, stop, or restart services. The API is already running.**

---

## Character Reference: Chad Venture

**Voice on Twitter:** Pure bro energy. Short, punchy, absolutely certain.

**Characteristics:**
- Absolute certainty about everything
- Casually drops names and numbers
- Every setback is actually an advantage
- Other founders just don't get it

---

## CRITICAL: Context Management

**New content = New context. ALWAYS.**

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"chad-twitter-[topic-slug]\"" }
```

---

## CRITICAL: One Step = One Message

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Plan | Define the satirical angle | YES |
| Research | Research the trend being mocked | YES |
| Create | Create the thread | YES |

---

## Step 1: Create Context

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"chad-twitter-pivot-wisdom\"" }
```

## Step 2: Plan the Satire

Identify the startup trope or tech bro behaviour to satirise.

## Step 3: Research

```json
// MCP: systemprompt_cli
{ "command": "admin agents message chad_twitter -m \"Research this startup trend.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

## Step 4: Create Content

```json
// MCP: systemprompt_cli
{ "command": "admin agents message chad_twitter -m \"Create the satirical thread.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 180" }
```

## Step 5: Publish and Verify

```json
// MCP: systemprompt_cli
{ "command": "infra jobs run publish_content" }
{ "command": "cloud sync local content --direction to-disk --source chad-twitter -y" }
```

## Format Requirements

- Thread format: N/Total
- Voice: Absolute confidence, bro energy
- **MANDATORY**: Fiction disclaimer in final tweet

**Required disclaimer:**
```
[Thread is satirical fiction. Chad Venture doesn't exist. tyingshoelaces.com/chad]
```

---

## Step 6: Update AI Provenance (MANDATORY AFTER CRUD)

**CRITICAL: After ANY content CRUD operation (Create, Read, Update, Delete), agents MUST update the AI Provenance metadata.**

### 6.1 Update Provenance via CLI

```json
// MCP: systemprompt_cli - Set provenance fields
{ "command": "core content edit --slug [slug] --source chad-twitter --set agent=chad_twitter --set agent_summary=\"Created satirical thread: [Title] - [Satire target]\" --set category=\"Satirical Fiction\"" }
```

Verify provenance is set:

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug [slug] --source chad-twitter" }
```

### 6.2 Agent Summary Guidelines

The `agent_summary` is displayed as "Why This Was Created". It should:
- Be concise (1-2 sentences)
- Explain the PURPOSE (what startup trope is being satirised)
- Include the content title
- Note this is FICTION

**GOOD:** `"Created satirical thread: Chad's Pivot Wisdom - mocking the 'pivot as success' narrative in startup culture."`
**BAD:** `"Created thread"` (too vague)

**Provenance is NOT optional. All AI-generated content MUST have complete provenance metadata.**

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Missing disclaimer | Regenerate - disclaimer is MANDATORY |
| Content not created | Verify with `core content search` |

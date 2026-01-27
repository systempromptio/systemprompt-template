---
title: "Execution Model Playbook"
description: "Standard execution model for content creation agents."
keywords:
  - execution
  - agents
  - content
  - workflow
---

# Execution Model: One Tool Per Message

This document defines the standard execution model for content creation agents.

> **Help**: `{ "command": "playbook content" }` via `systemprompt_help`

---

## Core Constraint

**Each tool call is a SEPARATE message. NEVER combine tools in a single message or tool plan.**

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Plan | Socratic Dialogue | YES - refine goals |
| Research | Call research_content | YES - wait for response |
| Create | Call content_create | YES - wait for response |
| Image | Generate featured image | YES - wait for response |

---

## Why This Matters

1. **Context Clarity**: Each tool call receives a clean context without competing artifacts
2. **Error Recovery**: If research fails, you can retry without affecting creation
3. **Artifact Tracking**: Research returns a `research_id` that must be explicitly passed to creation
4. **Debugging**: Easier to identify which step failed when they're separate

---

## Correct Workflow

```
User: "Create a blog post about AI agents"
Agent: [Socratic questions to clarify goal]
User: [Provides answers]
Agent: "CONFIRMED: [goal summary]. Proceeding with research."
Agent: [Calls research_content] → Returns research_id
Agent: "Research complete. ID: abc123. Now creating content."
Agent: [Calls content_create with research_id: abc123]
Agent: "Content created. Slug: ai-agents-production"
```

---

## Incorrect Workflow (NEVER DO THIS)

```
User: "Create a blog post about AI agents"
Agent: "Planning both research and creation..."
Agent: [Calls research_content AND content_create together]
```

This pattern causes:
- Missing research data in content creation
- Unable to use research_id reference
- Harder to debug failures

---

## Standard Execution Pattern

### Step 1: Socratic Dialogue

Agent interrogates your content goal to clarify:
- Target audience
- Key points to cover
- Tone and style
- Technical depth

### Step 2: Goal Confirmation

Agent confirms the plan before proceeding:
```
"CONFIRMED: Writing a technical blog post about MCP server architecture
for Rust developers. Will cover the rmcp crate, tool implementation,
and deployment patterns. Proceeding with research."
```

### Step 3: Research (Separate Message)

```json
// MCP: content_manager
{
  "name": "research_content",
  "arguments": {
    "topic": "MCP server architecture",
    "content_type": "blog",
    "target_audience": "Rust developers"
  }
}
```

Wait for response. Extract `research_id` from result.

### Step 4: Create (Separate Message)

```json
// MCP: content_manager
{
  "name": "content_create",
  "arguments": {
    "research_id": "abc123",
    "title": "Building MCP Servers in Rust",
    "content_type": "blog"
  }
}
```

### Step 5: Image (Separate Message, Optional)

```json
// MCP: content_manager
{
  "name": "generate_image",
  "arguments": {
    "slug": "building-mcp-servers",
    "description": "Technical diagram of MCP architecture"
  }
}
```

---

## Flow Diagram

```
┌─────────────┐
│   User      │
│   Request   │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────────┐
│          Socratic Dialogue              │
│   (Clarify goals, no tool calls)        │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│          Goal Confirmation              │
│   "CONFIRMED: [summary]. Proceeding."   │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│     Research (SEPARATE MESSAGE)         │
│   Call: research_content                │
│   Returns: research_id                  │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│     Create (SEPARATE MESSAGE)           │
│   Call: content_create                  │
│   Input: research_id from above         │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│     Image (SEPARATE MESSAGE)            │
│   Call: generate_image                  │
│   (Optional)                            │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────┐
│   Result    │
│   Summary   │
└─────────────┘
```

---

## Agent Types and Specializations

| Agent | Specialty | Playbook |
|-------|-----------|----------|
| `content` | Content Marketing Strategist | `playbook content` |
| `blog` | Blog Content Specialist | `playbook blog` |
| `linkedin` | LinkedIn Professional Agent | `playbook linkedin` |
| `medium` | Medium Narrative Agent | - |
| `hackernoon` | HackerNoon Technical Agent | - |
| `twitter` | Twitter/X Agent | `playbook twitter` |

Each agent follows the same execution model but with platform-specific adaptations.

---

## Research Skill Rules

**CRITICAL: Research skills must only be called ONCE per content piece.**

After research is complete:
1. You receive an `artifact_id` (research_id)
2. Use that ID when calling `content_create` in a SEPARATE message
3. DO NOT call research again unless explicitly instructed
4. If you need more information, reference the existing research

---

## Error Handling

If a step fails:

1. **Research fails**: Report error, ask user if they want to retry or adjust parameters
2. **Create fails**: Report error with research_id, ask if user wants to retry with same research
3. **Image fails**: Report error, content is still created, image can be added later

Never continue to the next step if the current step failed.

---

## Quick Reference

| Step | Tool | Input | Output |
|------|------|-------|--------|
| Research | `research_content` | topic, audience, type | `research_id` |
| Create | `content_create` | `research_id`, title, type | slug, content |
| Image | `generate_image` | slug, description | image_url |

| Rule | Description |
|------|-------------|
| One tool per message | Never combine research and create |
| Wait for response | Always wait for tool result before next step |
| Pass research_id | Explicitly pass ID from research to create |
| Confirm before proceeding | "CONFIRMED: [summary]. Proceeding with research." |

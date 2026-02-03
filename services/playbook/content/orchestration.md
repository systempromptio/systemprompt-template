---
title: "Blog Orchestration Playbook"
description: "Use the Blog Orchestrator to coordinate blog creation through the multi-agent mesh. Routes to specialised agents based on content type."
keywords:
  - blog
  - orchestration
  - agents
  - mesh
  - workflow
category: content
---

# Blog Orchestration Playbook

Use the Blog Orchestrator to coordinate blog creation through the multi-agent mesh. The orchestrator routes requests to specialised blog agents based on content type.

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.** Verify your session and profile before starting.

```json
{ "command": "core playbooks show cli_session" }
```

**Verify the mesh is running:**

```json
{ "command": "admin agents list --enabled" }
```

Required agents:
- `systemprompt_hub` - Central communications (Port 9020)
- `blog_orchestrator` - Workflow coordination (Port 9030)
- `blog_technical` - Technical deep-dives (Port 9040)
- `blog_narrative` - Personal narratives (Port 9050)

---

## Architecture Overview

```
User Request -> Blog Orchestrator -> Routes to -> Blog Agent (Technical or Narrative)
                     |                                    |
                     v                                    v
               SystemPrompt Hub                    Soul MCP Tools
               (Discord notifications)            (research, create, memory)
```

---

## Content Routing

The orchestrator routes based on content type:

| Content Type | Routed To | Keywords |
|--------------|-----------|----------|
| Technical deep-dive | `blog_technical` | architecture, contrarian, analysis, "why X is wrong", deep-dive |
| Personal narrative | `blog_narrative` | story, lessons, tutorial, "how I", personal, journey |

**Default:** If content type is unclear, routes to `blog_narrative`.

---

## Step 1: Send Request to Orchestrator

Send your blog request to the orchestrator. Be specific about the topic and angle.

### For Technical Deep-Dives

```json
{ "command": "admin agents message blog_orchestrator -m \"Create a TECHNICAL blog post about [topic].\n\nAngle: [contrarian angle - what conventional wisdom are you challenging?]\nEvidence: [what production experience or data backs this?]\nReader outcome: [what will readers do differently?]\" --blocking --timeout 600" }
```

**Example:**
```json
{ "command": "admin agents message blog_orchestrator -m \"Create a TECHNICAL blog post about MCP architecture.\n\nAngle: Challenge the assumption that complex multi-agent systems are necessary. Argue that well-designed context curation beats agent proliferation.\nEvidence: Production experience building the SystemPrompt mesh. Metrics on latency and reliability.\nReader outcome: Engineers will reconsider their multi-agent architectures and evaluate simpler alternatives.\" --blocking --timeout 600" }
```

### For Personal Narratives

```json
{ "command": "admin agents message blog_orchestrator -m \"Create a NARRATIVE blog post about [topic].\n\nStory: [the personal journey or experience]\nLesson: [what you learned]\nReader outcome: [what will readers take away?]\" --blocking --timeout 600" }
```

**Example:**
```json
{ "command": "admin agents message blog_orchestrator -m \"Create a NARRATIVE blog post about building the agent mesh.\n\nStory: The journey from monolithic agent to distributed mesh. What failed, what worked.\nLesson: Specialisation beats generalisation for production AI systems.\nReader outcome: Teams will understand when to split agents and when to keep them unified.\" --blocking --timeout 600" }
```

---

## Step 2: Monitor Progress

The orchestrator will:
1. Notify SystemPrompt Hub (sends Discord notification)
2. Create a shared context for the workflow
3. Route to the appropriate blog agent
4. Wait for completion (up to 5 minutes)
5. Notify Hub on completion or failure

### Check Hub for Updates

```json
{ "command": "admin agents message systemprompt_hub -m \"What's the latest workflow status?\" --blocking" }
```

### Check Agent Logs

```json
{ "command": "admin agents logs blog_orchestrator -n 50" }
{ "command": "admin agents logs blog_technical -n 50" }
{ "command": "admin agents logs blog_narrative -n 50" }
```

---

## Step 3: Verify and Publish

After the orchestrator completes, verify and publish the content.

### Verify Content Exists

```json
{ "command": "core content show [slug] --source blog" }
```

### Sync to Filesystem

```json
{ "command": "cloud sync local content --direction to-disk --source blog -y" }
```

### Publish

```json
{ "command": "infra jobs run publish_pipeline" }
```

---

## Direct Agent Access

You can bypass the orchestrator and message blog agents directly:

### Technical Agent

```json
{ "command": "admin agents message blog_technical -m \"Create a technical blog post about [topic].\n\nAngle: [contrarian angle]\nEvidence: [backing data/experience]\nKeywords: [keyword1, keyword2, keyword3]\" --blocking --timeout 300" }
```

### Narrative Agent

```json
{ "command": "admin agents message blog_narrative -m \"Create a narrative blog post about [topic].\n\nStory: [personal journey]\nLesson: [key takeaway]\nKeywords: [keyword1, keyword2, keyword3]\" --blocking --timeout 300" }
```

---

## Workflow Notifications

The SystemPrompt Hub sends Discord notifications for:
- `WORKFLOW_START` - When orchestrator begins
- `WORKFLOW_COMPLETE` - When blog is published (includes slug)
- `WORKFLOW_FAILED` - When something goes wrong (includes reason)

### Test Notifications

```json
{ "command": "admin agents message systemprompt_hub -m \"TEST: Verify Discord notification\" --blocking" }
```

---

## Agent Capabilities

### blog_technical

- **Skills:** edwards_voice, technical_content_writing, research_blog, blog_image_generation
- **MCP:** soul (memory, blog tools)
- **Output:** 4000-6000 words
- **Structure:** Orthodoxy -> Cracks -> Deeper Truth -> Implications

### blog_narrative

- **Skills:** edwards_voice, blog_writing, research_blog, blog_image_generation
- **MCP:** soul (memory, blog tools)
- **Output:** 3500-5000 words
- **Structure:** Prelude -> Problem -> Journey -> Lesson -> Conclusion

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Orchestrator timeout | Increase timeout: `--timeout 900` |
| Wrong agent selected | Be explicit: "Create a TECHNICAL blog" or "Create a NARRATIVE blog" |
| Hub not notifying | Check Discord config: `admin config show discord` |
| Blog agent fails | Check logs: `admin agents logs blog_technical -n 100` |
| Content not created | Verify with: `core content search "[topic]" --source blog` |

### Check Agent Registry

```json
{ "command": "admin agents registry" }
```

### Restart Agents

```json
{ "command": "infra services restart --agents" }
```

---

## Quick Reference

| Task | Command |
|------|---------|
| Create technical blog | `admin agents message blog_orchestrator -m "Create a TECHNICAL blog..." --blocking --timeout 600` |
| Create narrative blog | `admin agents message blog_orchestrator -m "Create a NARRATIVE blog..." --blocking --timeout 600` |
| Check workflow status | `admin agents message systemprompt_hub -m "Latest workflow status?" --blocking` |
| View agent logs | `admin agents logs [agent_name] -n 50` |
| Verify content | `core content show [slug] --source blog` |
| Publish content | `infra jobs run publish_pipeline` |
| Test notifications | `admin agents message systemprompt_hub -m "TEST: notification" --blocking` |

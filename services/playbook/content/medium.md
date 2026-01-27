---
title: "Medium Content Creation Playbook"
description: "Create story-driven Medium articles that make technical insights resonate emotionally with broader audiences."
keywords:
  - medium
  - storytelling
  - emotional
  - technical
---

# Medium Content Creation Playbook

Create story-driven Medium articles that make technical insights resonate emotionally with broader audiences.

## Prerequisites

**Load the [Session Playbook](../cli/session.md) first.** Verify your session and profile before starting.

```json
// MCP: systemprompt_help
{ "command": "playbook session" }
```

**IMPORTANT: NEVER start, stop, or restart services. The API is already running.**

```json
// MCP: systemprompt_cli
{ "command": "admin session show" }
```

```json
// MCP: systemprompt_cli
{ "command": "admin agents list --enabled" }
```

If services are not running, ask the user to start them. Do not attempt to start them yourself.

---

## MCP Tools

| Tool | Purpose |
|------|---------|
| `systemprompt_cli` | Execute CLI commands |
| `systemprompt_help` | Get command documentation and playbooks |

---

## CRITICAL: Context Management

**New content = New context. ALWAYS.** See the [Contexts Playbook](../cli/contexts.md) for full context management commands.

**WRONG:** Reusing `medium-general` context for multiple articles
**RIGHT:** `medium-open-source-belief`, `medium-ai-burnout-story`, `medium-startup-lessons` (one context per article)

---

## CRITICAL: One Step = One Message

**Each step in this playbook is a SEPARATE message to the agent. NEVER combine steps.**

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Step 3 | Plan | YES - send message, wait for response |
| Step 4 | Research | YES - send message, wait for response |
| Step 5 | Create | YES - send message, wait for response |
| Step 6 | Image | YES - send message, wait for response |

**WRONG:** "Research and create an article about X"
**RIGHT:** Message 1: "Research X" → wait → Message 2: "Create the article"

---

## Step 1: Create a FRESH Context (REQUIRED FIRST STEP)

**Every new article MUST start with a fresh context.** Name it descriptively for what you're creating.

```json
// MCP: systemprompt_cli
{ "command": "core contexts list" }
```

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"medium-[topic-slug]\"" }
```

**Examples of good context names:**
```json
{ "command": "core contexts new --name \"medium-open-source-belief\"" }
{ "command": "core contexts new --name \"medium-ai-burnout-story\"" }
{ "command": "core contexts new --name \"medium-startup-pivot-lessons\"" }
```

**ONLY continue an existing context if:**
- You're resuming work on the SAME article
- You haven't created any other content since
- The context was created specifically for this article

```json
// MCP: systemprompt_cli - ONLY if continuing the SAME article
{ "command": "core contexts use medium-open-source-belief" }
```

## Step 2: Review Performance and Trends

```json
// MCP: systemprompt_cli
{ "command": "analytics content stats --since 30d" }
```

```json
// MCP: systemprompt_cli
{ "command": "analytics content top --since 30d --limit 10" }
```

Share insights:

```json
// MCP: systemprompt_cli
{ "command": "admin agents message medium -m \"Our top performing articles are about [topics]. What angles haven't we explored?\" --blocking" }
```

**WAIT for response. Save the CONTEXT_ID from the response for all subsequent messages.**

## Step 3: SOCRATIC DIALOGUE - Convince the Gatekeeper (MANDATORY)

**CRITICAL: The Medium agent is a GATEKEEPER. It will NOT research or create content until you have convinced it with deep, articulated reasoning about the EMOTIONAL VALUE.**

The agent uses the Socratic method to interrogate your content idea. Your job is to provide clear, specific, well-reasoned answers that demonstrate you understand the emotional resonance this story will create.

### 3.1 Initiate the Dialogue

```json
// MCP: systemprompt_cli
{ "command": "admin agents message medium -m \"I want to write a Medium article about [topic]. Let's define the emotional hook before researching.\" --context-id \"$CONTEXT_ID\" --blocking" }
```

The agent will ask probing questions. **DO NOT skip this step or give vague answers.**

### 3.2 Be Prepared to Answer These Questions

**On Emotional Hook:**
- What's the specific moment that made you feel something about this?
- If you had to make someone care in the first paragraph, what would you say?
- What's the emotional stakes - fear, hope, frustration, revelation?
- Why would someone who isn't technical still care about this?

**On Personal Story:**
- What specific failure or breakthrough anchors this story?
- What did you believe before that you no longer believe?
- What moment changed everything for you?
- What's the most vulnerable thing you could admit about this journey?

**On Transformation Arc:**
- How should the reader feel at the end vs the beginning?
- What realisation do you want them to arrive at themselves?
- What will they see differently after reading this?
- What's the emotional journey - from cynicism to hope? From confidence to humility?

**On Stakes:**
- Why does this story matter to someone outside tech?
- What's the universal human experience inside this technical topic?
- What would be lost if this story went untold?

### 3.3 Example Dialogue

**WRONG (Vague - Agent Will Reject):**
```json
// MCP: systemprompt_cli
{ "command": "admin agents message medium -m \"Write about open source\" --context-id \"$CONTEXT_ID\" --blocking" }
// Agent will push back: "What's the emotional hook? What personal story? What's the transformation?"
```

**RIGHT (Specific - Agent Will Accept):**
```json
// MCP: systemprompt_cli
{ "command": "admin agents message medium -m \"I want to write a Medium article about why I still believe in open source after 15 years.\n\nEMOTIONAL HOOK: That moment when I saw my abandoned side project being used in production by a Fortune 500 company - and feeling both pride and despair that I'd given up on it too soon.\n\nPERSONAL STORY: I maintained an open source project for 3 years with zero recognition. I burned out. I abandoned it. Two years later, I discovered it was being used by 400+ companies. The project I thought was a failure had become infrastructure.\n\nTRANSFORMATION ARC: Reader goes from feeling cynical about open source (thankless, exploitative, burnout factory) to understanding the slow, invisible ripples of contribution that you may never see.\n\nUNIVERSAL STAKES: This is about any work where the impact is invisible - teachers, parents, infrastructure engineers. The question: how do you sustain effort when you can't see the results?\n\nTITLE DIRECTION: 'The Project I Abandoned Became Infrastructure'\n\nLet's refine this before researching.\" --context-id \"$CONTEXT_ID\" --blocking" }
```

### 3.4 Gate Check - Agent Must Confirm

The agent will NOT proceed to research until it can articulate:
- The emotional hook in one sentence
- The specific personal story anchor
- The transformation arc (from X to Y)
- Why a non-technical person would still care

**Wait for the agent to confirm the goal:**
```
CONFIRMED MEDIUM CONTENT GOAL:
- Emotional hook: [the feeling that grabs readers]
- Personal story anchor: [specific moment/failure/breakthrough]
- Transformation arc: [reader goes from feeling X to feeling Y]
- Universal stakes: [why anyone would care]
- Title direction: [punchy, personal, intriguing]
```

**Only then will the agent proceed to research.**

---

## Step 4: Research the Topic (Agent-Initiated After Goal Confirmed)

Once the agent confirms the goal, ask it to research using `research_content`:

```json
// MCP: systemprompt_cli
{ "command": "admin agents message medium -m \"The goal is confirmed. Research this topic using research_content with content_type='medium_article'. Return the research_id when complete.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

**WAIT for response. Save the research_id (e.g., `4a7f8ac2-9220-4c63-88ee-d9ea2d73e7b7`).**

**NOTE:** Research may return "0 recent posts" - this is expected if Medium content isn't indexed. Continue to Step 5 anyway.

## Step 5: Create the Content (With Precise Guidance From Dialogue)

The agent will include everything from the Socratic dialogue in the `instructions` field automatically. You can add specifics:

```json
// MCP: systemprompt_cli
{ "command": "admin agents message medium -m \"Create the Medium article using content_create with:\n- research_id='$RESEARCH_ID'\n- slug='abandoned-project-became-infrastructure'\n- content_type='medium_article'\n- keywords=['open source', 'burnout', 'invisible impact', 'maintainer', 'infrastructure']\n\nAdditional instructions: Open with the Fortune 500 discovery moment. Build to the 400+ companies revelation. End on the question about invisible work. 2500-4000 words. British English.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 600" }
```

**Critical:** The agent will embed the confirmed emotional hook, personal story, and transformation arc from the Socratic dialogue into the content creation.

**IMPORTANT:** Use `--timeout 600` for long-form content creation.

**WAIT for response. Verify content exists even if CLI times out:**

```json
// MCP: systemprompt_cli
{ "command": "core content search \"abandoned-project-became-infrastructure\"" }
```

**If the agent says content was created but it doesn't exist:**
- Create a NEW context and start fresh
- The most common cause is context contamination from previous sessions

**Then proceed to Step 6.**

## Step 6: Generate Featured Image (SEPARATE MESSAGE)

```json
// MCP: systemprompt_cli
{ "command": "admin agents message medium -m \"Generate a featured image for this article representing [theme].\" --context-id \"$CONTEXT_ID\" --blocking --timeout 60" }
```

**WAIT for response.**

## Step 7: Publish and Verify

**IMPORTANT:** Image optimization MUST run before publish. Templates expect `.webp` images, but content is created with `.png`. Without optimization, featured images will 404.

See the [Jobs Playbook](../cli/jobs.md) for job management commands.

```json
// MCP: systemprompt_cli - Run image optimization FIRST (converts PNG to WebP)
{ "command": "infra jobs run blog_image_optimization" }
```

```json
// MCP: systemprompt_cli - Then run the publish job (prerenders, updates sitemap)
{ "command": "infra jobs run publish_content" }
```

```json
// MCP: systemprompt_cli - Sync content from database to filesystem (REQUIRED for new content)
{ "command": "cloud sync local content --direction to-disk --source medium -y" }
```

```json
// MCP: systemprompt_cli - Verify the content is accessible and published
{ "command": "core content verify --slug my-article-slug" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug my-article-slug" }
```

---

## MCP Tools Used by Agent

| Tool | Purpose | Notes |
|------|---------|-------|
| `research_content` | Topic research, returns `research_id` | Use with content_type='medium_article' |
| `content_create` | Create content from research | Requires valid research_id |
| `generate_image` | Generate AI image | Requires content_id |

## Content Requirements (MANDATORY - Will Be Rejected If Not Met)

| Requirement | Specification | Validation |
|-------------|---------------|------------|
| **Word count** | 2,500-4,000 words | Content < 2,500 will be rejected |
| **External citations** | 5+ with full URLs | Not counting tyingshoelaces.com backlinks |
| **Backlinks** | 2 to tyingshoelaces.com | One early, one closing |
| **Language** | British English ONLY | realise, optimise, colour, behaviour |
| **Signature** | "Ship first. Philosophise later." | Must appear at end |
| **Keywords** | 3-5 SEO keywords | Required for content_create |
| **Image** | Featured image | Placeholder if not generated |

### British English Reference

| American (WRONG) | British (CORRECT) |
|------------------|-------------------|
| realize | realise |
| optimize | optimise |
| color | colour |
| behavior | behaviour |
| favor | favour |
| organization | organisation |

### AI Anti-Patterns to Avoid

- "Let me explain..." / "Let's dive in..." / "Let's break down..."
- "Here's what I learned:" / "Here's the thing:"
- Opening with a rhetorical question
- Starting paragraphs with "First," "Second," "Third,"
- En-dashes (–) - use hyphens (-) instead

---

## Example Session (Each Step is Separate)

```json
// Step 1: Create context
{ "command": "core contexts new --name \"medium-open-source-belief\"" }

// Step 2: Review trends and get CONTEXT_ID
{ "command": "analytics content top --since 30d --limit 5" }

// First message - capture the context ID from response
{ "command": "admin agents message medium -m \"Our top articles are about AI production. What angles about open source haven't we explored?\" --blocking --json" }
// Save CONTEXT_ID from response

// Step 3: Plan (SEND, WAIT FOR RESPONSE)
{ "command": "admin agents message medium -m \"I want to write about why I still believe in open source after 15 years. What's the emotional hook? What personal failure can anchor this?\" --context-id \"$CONTEXT_ID\" --blocking" }

// Step 4: Research (SEPARATE MESSAGE, WAIT FOR RESPONSE)
{ "command": "admin agents message medium -m \"Research open source sustainability and maintainer burnout. Return the research_id when done.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
// Save the research_id from response, e.g.: RESEARCH_ID="4a7f8ac2-9220-4c63-88ee-d9ea2d73e7b7"

// Step 5: Create (SEPARATE MESSAGE, WAIT FOR RESPONSE)
{ "command": "admin agents message medium -m \"Create the article with:\n- research_id='$RESEARCH_ID'\n- slug='why-i-still-believe-in-open-source'\n- keywords=['open source', 'maintainer burnout', 'software sustainability', 'community']\n\nFocus on the journey from cynicism to conviction. Use the hook we discussed about the abandoned project.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 180" }

// VERIFY content exists - CRITICAL STEP
{ "command": "core content search \"why-i-still-believe-in-open-source\"" }

// Step 6: Image (SEPARATE MESSAGE, WAIT FOR RESPONSE)
{ "command": "admin agents message medium -m \"Generate a featured image representing hope in open source\" --context-id \"$CONTEXT_ID\" --blocking" }

// Step 7: Optimize images, Publish, and Sync to filesystem
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run publish_content" }
{ "command": "cloud sync local content --direction to-disk --source medium -y" }
{ "command": "core content verify --slug why-i-still-believe-in-open-source" }
```

---

## Step 8: Validate Filesystem and Refine Content (FINAL QUALITY GATE)

After publishing, validate the content has been synced to the filesystem and perform a final quality review.

### 8.1 Validate Filesystem Sync

See the [Content Playbook](../cli/content.md) for content management commands.

**IMPORTANT:** Content created by agents lives in the database. You MUST sync to filesystem before validating:

```json
// MCP: systemprompt_cli - Sync database content to filesystem
{ "command": "cloud sync local content --direction to-disk --source medium -y" }
```

```json
// MCP: systemprompt_cli - Verify sync status
{ "command": "core content status --source medium" }
```

**Expected frontmatter fields:**
```yaml
---
title: "Article Title"
description: "Brief description"
author: "Edward"
slug: "why-i-still-believe-in-open-source"
keywords: "keyword1, keyword2, keyword3"
image: "/files/images/..."
kind: "medium"
public: true
tags: []
published_at: "2026-01-17"
---
```

### 8.2 Review Against Original Aim

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug why-i-still-believe-in-open-source --source medium" }
```

**Evaluation checklist:**
- [ ] Hook captures the emotional angle agreed in planning
- [ ] Personal story anchors the narrative
- [ ] Length meets requirement (2,500-4,000 words)
- [ ] Minimum 5 grounding reference links (inline citations)
- [ ] British English (realise, optimise)
- [ ] Structure follows story arc (hook → journey → insight → call-to-action)
- [ ] No AI anti-patterns ("I discovered that...", "Fascinatingly...")
- [ ] Authentic voice - personal failures, honest takes

### 8.3 Edit to World-Class Quality

If the content needs refinement, edit directly in the filesystem, then re-ingest:

```json
// MCP: systemprompt_cli
{ "command": "core content ingest services/content/medium --source medium --override" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug why-i-still-believe-in-open-source --source medium" }
```

**Refinement focus areas:**
- Strengthen the opening hook - it must grab within first paragraph
- Ensure personal story feels authentic, not manufactured
- Add specific details: dates, numbers, names (where appropriate)
- Check each section earns its place in the narrative arc
- Verify citations are inline with descriptive anchor text, not domain-only
- Remove any generic conclusions or hollow engagement questions
- Tighten transitions between sections

### 8.4 Final Verification

```json
// MCP: systemprompt_cli
{ "command": "core content verify --slug why-i-still-believe-in-open-source --source medium --base-url https://tyingshoelaces.com" }
```

**Content is ONLY complete when:**
1. ✓ Exists in database (`core content show` succeeds)
2. ✓ Exists on filesystem (`services/content/medium/[slug]/index.md` exists)
3. ✓ Frontmatter is valid YAML with all required fields
4. ✓ Word count is 2,500-4,000 words
5. ✓ Contains minimum 5 inline citations with full URLs
6. ✓ Matches the emotional hook and personal narrative agreed in planning
7. ✓ Published and accessible on site

---

## Step 9: Update AI Provenance (MANDATORY AFTER CRUD)

**CRITICAL: After ANY content CRUD operation (Create, Read, Update, Delete), agents MUST update the AI Provenance metadata.**

AI Provenance is rendered in the content template as a transparency block showing readers how the content was created.

### 9.1 What AI Provenance Displays

The template renders an "AI Provenance" section showing:

| Field | Description | Example |
|-------|-------------|---------|
| Agent | Agent that created/modified content | `Medium Agent @medium` |
| Why This Was Created | Agent's summary of purpose | "Created article: The Project I Abandoned - exploring invisible impact" |
| Category | Content category | Personal Story, Technical Journey |
| Platform | Target platform | medium |
| Created | Original creation date | Jan 26, 2026 |

### 9.2 Provenance Fields in Frontmatter

```yaml
---
# ... other fields ...
agent: "medium"
agent_summary: "Created article: [Title] - [Brief description of why]"
category: "Personal Story"
platform: "medium"
created_at: "2026-01-26"
updated_at: "2026-01-27"  # If updated
---
```

### 9.3 When to Update Provenance

| Operation | Action Required |
|-----------|-----------------|
| **Create** | Set all provenance fields. `agent_summary` explains WHY this content was created. |
| **Update** | Update `updated_at` and append to `agent_summary` explaining what changed. |
| **Delete** | No action (content removed). |

### 9.4 Agent Summary Guidelines

The `agent_summary` is displayed as "Why This Was Created". It should:
- Be concise (1-2 sentences)
- Explain the PURPOSE, not just the action
- Include the content title
- Reference the emotional hook if relevant

**GOOD:** `"Created article: Why I Still Believe in Open Source - exploring invisible impact after 15 years."`
**BAD:** `"Created article"` (too vague)

### 9.5 Update Provenance After Operations

```json
// MCP: systemprompt_cli - Set provenance fields
{ "command": "core content edit --slug [slug] --source medium --set agent=medium --set agent_summary=\"Created article: [Title] - [Why]\" --set category=\"Personal Story\"" }
```

Verify provenance is set:

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug [slug] --source medium" }
```

**Provenance is NOT optional. All AI-generated content MUST have complete provenance metadata.**

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Agent not responding | Check with `{ "command": "infra services status" }` |
| Content not created | Verify with `{ "command": "core content search \"[slug]\"" }`. Agent may claim success falsely. |
| Agent says "created" but content doesn't exist | **Create a NEW context** and try again. Context contamination is the most common cause. |
| Research artifact not found | **Create a NEW context**. The old context has conflicting artifacts. |
| Agent uses wrong research_id | **Create a NEW context**. Multiple research artifacts are confusing the agent. |
| Agent hallucinates Content ID | **Create a NEW context**. Agent is referencing artifacts from previous sessions. |
| Content doesn't match planning | Include planning details (hook, arc) in the create message explicitly |
| Context lost between messages | Always use `--context-id "$CONTEXT_ID"` for every message |
| Missing image | Send separate image generation message |
| "Keywords cannot be empty" error | Include keywords array in content_create message |
| Agent re-researches instead of creating | Provide explicit research_id: "Use research_id='X' DO NOT call any research tool" |
| Generated content too short | Explicitly state "MUST be 2500-4000 words minimum" |
| CLI timeout on content_create | Use `--timeout 600`. Check if content was created: `core content search` |
| Gemini 503 errors | Retry after 10-30 seconds. Check `logs/mcp-content-manager.log` |
| Research returns 0 posts | Expected - Medium content not yet indexed. Continue with content_create anyway. |

### Context-Related Failures

See the [Session Playbook](../cli/session.md#context-management-quick-reference) and [Contexts Playbook](../cli/contexts.md) for context contamination symptoms and solutions. The fix is always: **create a NEW context and start fresh.**

---

## Known Issues (2026-01-17)

See `plan/feedback/` for full details on each issue.

### BLOCKING Issues

| Issue | Status | Impact | Feedback File |
|-------|--------|--------|---------------|
| **research_content empty Gemini response** | BLOCKING | Cannot complete research step | `research-gemini-empty-response.md` |
| **content_create requires research_id** | BLOCKING | Cannot skip research | `content-create-requires-research-id.md` |

### Non-Blocking Issues

1. **Research returns 0 posts** - Medium content not indexed. Does not block creation.
2. **Agent may ignore research_id** - Always say "Use EXACTLY this research_id, DO NOT call any research tool"
3. **CLI timeout** - Use `--timeout 600` for content_create. Verify with search after timeout.

### Workaround for Blocked Playbook

Until Gemini API issues are resolved, content creation workflow is blocked. Workarounds:

1. **Manual content creation** - Write directly to `services/content/medium/` and ingest
2. **Wait for Gemini API** - Retry research tools after 30-60 seconds
3. **Check API credentials** - Verify Gemini API key in profile configuration

---

## Recommended Timeouts

| Step | Timeout |
|------|---------|
| Research | `--timeout 120` |
| Content Create | `--timeout 600` |
| Image Generate | `--timeout 60` |
| Planning | `--timeout 60` |

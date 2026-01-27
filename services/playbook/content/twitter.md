---
title: "Twitter/X Content Creation Playbook"
description: "Create punchy, contrarian content armed with production reality that cuts through the hype. Sharp, direct, data-backed, substance over virality."
keywords:
  - twitter
  - threads
  - contrarian
  - viral
---

# Twitter/X Content Creation Playbook

Create punchy, contrarian content armed with production reality that cuts through the hype. Sharp, direct, data-backed, substance over virality.

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

**WRONG:** Reusing `twitter-general` context for multiple threads
**RIGHT:** `twitter-ai-costs-thread`, `twitter-rag-reality`, `twitter-agent-myths` (one context per content piece)

---

## CRITICAL: One Step = One Message

**Each step in this playbook is a SEPARATE message to the agent. NEVER combine steps.**

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Step 3 | Plan | YES - send message, wait for response |
| Step 4 | Research | YES - send message, wait for response |
| Step 5 | Create | YES - send message, wait for response |

**WRONG:** "Research and create a thread about X"
**RIGHT:** Message 1: "Research X" → wait → Message 2: "Create the thread"

---

## Step 1: Create a FRESH Context (REQUIRED FIRST STEP)

**Every new piece of content MUST start with a fresh context.** Name it descriptively for what you're creating.

```json
// MCP: systemprompt_cli
{ "command": "core contexts list" }
```

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"twitter-[topic-slug]\"" }
```

**Examples of good context names:**
```json
{ "command": "core contexts new --name \"twitter-ai-agent-costs\"" }
{ "command": "core contexts new --name \"twitter-rag-production-fails\"" }
{ "command": "core contexts new --name \"twitter-mcp-reality-check\"" }
```

**ONLY continue an existing context if:**
- You're resuming work on the SAME content piece
- You haven't created any other content since
- The context was created specifically for this content

```json
// MCP: systemprompt_cli - ONLY if continuing the SAME content piece
{ "command": "core contexts use twitter-ai-agent-costs" }
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

Share insights and get CONTEXT_ID:

```json
// MCP: systemprompt_cli - First message, capture the context ID from response
{ "command": "admin agents message twitter -m \"Our top threads are about [topics]. What contrarian take should we explore?\" --blocking --json" }
```

**Save the CONTEXT_ID for all subsequent messages.**

## Step 3: SOCRATIC DIALOGUE - Convince the Gatekeeper (MANDATORY)

**CRITICAL: The Twitter agent is a GATEKEEPER. It will NOT research or create content until you have convinced it with deep, articulated reasoning about the VIRAL VALUE.**

The agent uses the Socratic method to interrogate your content idea. Your job is to provide clear, specific, well-reasoned answers that demonstrate you understand what makes content spread on Twitter.

### 3.1 Initiate the Dialogue

```json
// MCP: systemprompt_cli
{ "command": "admin agents message twitter -m \"I want to create a Twitter thread about [topic]. Let's define the contrarian take before researching.\" --context-id \"$CONTEXT_ID\" --blocking" }
```

The agent will ask probing questions. **DO NOT skip this step or give vague answers.**

### 3.2 Be Prepared to Answer These Questions

**On Contrarian Take:**
- What's the conventional wisdom you're challenging?
- Who will disagree with this, and why are they wrong?
- What does everyone believe that isn't true?
- What uncomfortable truth are you revealing?

**On Viral Hook:**
- Can tweet 1 work completely standalone? Would someone RT just that?
- What's the 280-character version that would stop someone mid-scroll?
- If someone only sees the first tweet, do they get the punch?
- What's the specific, bold claim that grabs attention?

**On Evidence:**
- What production data backs this up?
- What specific numbers, metrics, or outcomes can you cite?
- What did you learn from doing, not just reading?
- What failure gave you this insight?

**On Insight Arc:**
- What's the journey from tweet 1 to the final tweet?
- What realisation should unfold across the thread?
- What's the turn - the moment where the reader's assumption breaks?
- How does each tweet earn its place in the sequence?

### 3.3 Example Dialogue

**WRONG (Vague - Agent Will Reject):**
```json
// MCP: systemprompt_cli
{ "command": "admin agents message twitter -m \"Write a thread about AI\" --context-id \"$CONTEXT_ID\" --blocking" }
// Agent will push back: "What's the contrarian take? What's the viral hook? What evidence?"
```

**RIGHT (Specific - Agent Will Accept):**
```json
// MCP: systemprompt_cli
{ "command": "admin agents message twitter -m \"I want to create a Twitter thread debunking AI productivity claims.\n\nCONTRARIAN TAKE: Companies claiming '10x developer productivity with AI' are measuring the wrong thing. They're counting lines of code, not shipped features. AI makes you type faster, not ship faster.\n\nVIRAL HOOK (Tweet 1): 'We gave our team GitHub Copilot. Lines of code went up 300%. Shipped features went down 15%. Here's what actually happened.'\n\nEVIDENCE: We measured this across 3 engineering teams for 6 months. LOC increased 300%. PR review time increased 200%. Bug rate increased 40%. Net velocity decreased 15% because developers spent more time reviewing AI-generated code than they saved writing it.\n\nINSIGHT ARC:\n1. Hook with the paradox (metrics up, output down)\n2. What we expected vs what happened\n3. The hidden cost: review time\n4. The bug rate surprise\n5. Why LOC is a terrible metric\n6. What actually predicts productivity\n7. What we changed\n8. CTA with link\n\nLet's refine this before researching.\" --context-id \"$CONTEXT_ID\" --blocking" }
```

### 3.4 Gate Check - Agent Must Confirm

The agent will NOT proceed to research until it can articulate:
- The contrarian take in one sentence
- Tweet 1 hook that works standalone
- The production evidence
- The insight arc (problem → turn → data → implications)

**Wait for the agent to confirm the goal:**
```
CONFIRMED TWITTER CONTENT GOAL:
- Contrarian take: [what conventional wisdom we're challenging]
- Tweet 1 hook: [standalone viral hook, under 280 chars]
- Evidence base: [specific production data/metrics]
- Insight arc: [problem → turn → data → implications]
- Who this challenges: [who will be uncomfortable]
- Thread length: [X tweets]
```

**Only then will the agent proceed to research.**

---

## Step 4: Research the Topic (Agent-Initiated After Goal Confirmed)

Once the agent confirms the goal, ask it to research:

```json
// MCP: systemprompt_cli
{ "command": "admin agents message twitter -m \"The goal is confirmed. Research this topic using research_content with content_type='twitter_thread'. Return the research_id when done.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

**WAIT for response. Save the research_id (e.g., `RESEARCH_ID="..."`). Then proceed to Step 5.**

## Step 5: Create the Content (With Precise Guidance From Dialogue)

The agent will include everything from the Socratic dialogue in the `instructions` field automatically. You can add specifics:

```json
// MCP: systemprompt_cli
{ "command": "admin agents message twitter -m \"Create the Twitter thread using content_create with:\n- research_id='$RESEARCH_ID'\n- slug='ai-productivity-paradox'\n- content_type='twitter_thread'\n- keywords=['AI', 'productivity', 'Copilot', 'metrics', 'engineering']\n\nAdditional instructions: Use the exact hook we discussed about 300%/15%. Include all 8 tweets in the arc. Reference the 6-month study and the 40% bug rate increase.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

**Critical:** The agent will embed the confirmed contrarian take, viral hook, and insight arc from the Socratic dialogue into the content creation.

**WAIT for response. The agent should report:**
- Content ID (e.g., `581ffe8b-63ae-4a2b-9221-23c304d75fc2`)
- URL (e.g., `/ai-productivity-paradox`)
- Word count

**CRITICAL: If the agent does NOT report a Content ID, the creation FAILED. Do not proceed.**

```json
// MCP: systemprompt_cli - Verify content exists in database
{ "command": "core content search \"ai-productivity-paradox\"" }
```

```json
// MCP: systemprompt_cli - View full content to check quality
{ "command": "core content show <content-id>" }
```

## Step 6: Quality Check and Publish

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
{ "command": "cloud sync local content --direction to-disk --source twitter -y" }
```

```json
// MCP: systemprompt_cli - Verify the content is accessible and published
{ "command": "core content verify --slug ai-hype-reality" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug ai-hype-reality" }
```

---

## Content Types

| Type | Format | Use Case |
|------|--------|----------|
| `twitter_post` | Single tweet (280 chars) | Quick takes |
| `twitter_thread` | 5-12 tweets | Deep insights |

## MCP Tools Used by Agent

| Tool | Purpose |
|------|---------|
| `research_content` | Research topic, returns `research_id` |
| `content_create` | Create content from research |
| `analytics_query` | Query performance data |

## Format Requirements

**Single Tweet (280 chars):**
- Hook + insight + punch
- No links

**Thread (5-12 tweets):**
- Number as N/Total: 1/8, 2/8, 3/8...
- Tweet 1: Hook that works standalone
- Tweets 2-3: Context setup
- Tweets 4-8: Insights with data
- Final: CTA with link

**Required signature (final tweet):**
```
Now if you'll excuse me, I'm off to build stuff.

https://tyingshoelaces.com/blog/[slug]
```

**Forbidden:** Hashtags, emojis, em-dashes, links in first tweet

---

## Example Session (Each Step is Separate)

```json
// Step 1: Create context
{ "command": "core contexts new --name \"twitter-ai-replacement\"" }

// Step 2: Review trends and get CONTEXT_ID
{ "command": "analytics content top --since 30d --limit 5" }

// First message - capture the context ID from response
{ "command": "admin agents message twitter -m \"Our top threads are debunking AI hype. What angle haven't we covered?\" --blocking --json" }
// Save CONTEXT_ID from response

// Step 3: Plan (SEND, WAIT FOR RESPONSE)
{ "command": "admin agents message twitter -m \"I want to write a thread debunking 'AI will replace developers'. What's the viral hook for tweet 1?\" --context-id \"$CONTEXT_ID\" --blocking" }

// Step 4: Research (SEPARATE MESSAGE, WAIT FOR RESPONSE)
{ "command": "admin agents message twitter -m \"Research AI developer productivity claims and real production adoption data. Return the research_id when done.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
// Save the research_id from response, e.g.: RESEARCH_ID="4a7f8ac2-..."

// Step 5: Create (SEPARATE MESSAGE, WAIT FOR RESPONSE)
{ "command": "admin agents message twitter -m \"Create a Twitter thread with:\n- research_id='$RESEARCH_ID'\n- slug='ai-developer-replacement-reality'\n- content_type='twitter_thread'\n- keywords=['AI', 'developers', 'replacement', 'productivity', 'reality']\n\n8 tweets. Use the hook we discussed.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }

// VERIFY content exists - CRITICAL STEP
{ "command": "core content search \"ai-developer-replacement-reality\"" }

// Step 6: Optimize images and Publish
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run publish_content" }
{ "command": "core content verify --slug ai-developer-replacement-reality" }
```

---

## Thread Structure Template

```
1/8 [HOOK - must work standalone]
2/8 [Context - conventional wisdom]
3/8 [The turn - what we discovered]
4/8 [Data point 1]
5/8 [Data point 2]
6/8 [Insight]
7/8 [What this means]
8/8 [Signature + CTA]
```

---

## Quality Checklist

Before publishing, verify the thread meets these criteria:

| Criterion | How to Check |
|-----------|--------------|
| N/Total format | First tweet starts with "1/8" (or similar) |
| Standalone hook | Tweet 1 makes sense without context |
| No hashtags | Search for "#" in content |
| No emojis | Visual inspection |
| Correct signature | Ends with "Now if you'll excuse me..." |
| URL matches slug | Final URL includes the slug |
| 8 tweets | Count the numbered tweets |

---

## Step 7: Validate Filesystem and Refine Content (FINAL QUALITY GATE)

After publishing, validate the content has been synced to the filesystem and perform a final quality review.

### 7.1 Validate Filesystem Sync

See the [Content Playbook](../cli/content.md) for content management commands.

```json
// MCP: systemprompt_cli
{ "command": "core content status --source twitter" }
```

**Expected frontmatter fields:**
```yaml
---
title: "Thread Title"
description: "Brief description"
author: "Edward"
slug: "ai-developer-replacement-reality"
keywords: "keyword1, keyword2"
image: "/files/images/..."
kind: "twitter"
public: true
tags: []
published_at: "2026-01-17"
---
```

### 7.2 Review Against Original Aim

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug ai-developer-replacement-reality --source twitter" }
```

**Evaluation checklist (threads):**
- [ ] Tweet 1 works completely standalone (the viral hook)
- [ ] Numbering format is N/Total (e.g., 1/8, 2/8, not 1/, 2/)
- [ ] Thread has 5-12 tweets total
- [ ] No hashtags anywhere
- [ ] No emojis anywhere
- [ ] No links in tweet 1 (hook only)
- [ ] Final tweet has correct signature: "Now if you'll excuse me, I'm off to build stuff."
- [ ] Final tweet has correct URL: `https://tyingshoelaces.com/blog/[slug]`
- [ ] Each tweet is under 280 characters
- [ ] Insight arc builds (context → turn → data → implications)

**Evaluation checklist (single tweets):**
- [ ] Under 280 characters
- [ ] Hook + insight + punch structure
- [ ] No hashtags, no emojis
- [ ] No links (standalone tweets)

### 7.3 Edit to World-Class Quality

If the content needs refinement, edit directly in the filesystem, then re-ingest:

```json
// MCP: systemprompt_cli
{ "command": "core content ingest services/content/twitter --source twitter --override" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug ai-developer-replacement-reality --source twitter" }
```

**Refinement focus areas:**
- Strengthen tweet 1 - it must work completely standalone and hook readers
- Ensure each tweet adds value (no filler)
- Check data points are specific and credible
- Verify the "turn" in tweet 3 surprises or challenges
- Remove any generic phrases or weak conclusions
- Ensure punch lands in every tweet

### 7.4 Final Verification

```json
// MCP: systemprompt_cli
{ "command": "core content verify --slug ai-developer-replacement-reality --source twitter --base-url https://tyingshoelaces.com" }
```

**Content is ONLY complete when:**
1. ✓ Exists in database (`core content show` succeeds)
2. ✓ Exists on filesystem (`services/content/twitter/[slug]/index.md` exists)
3. ✓ Frontmatter is valid YAML with all required fields
4. ✓ Thread structure is correct (N/Total numbering, proper tweet count)
5. ✓ No forbidden elements (hashtags, emojis, links in wrong places)
6. ✓ Matches the contrarian angle and hook agreed in planning
7. ✓ Published and accessible on site

---

## Step 8: Update AI Provenance (MANDATORY AFTER CRUD)

**CRITICAL: After ANY content CRUD operation (Create, Read, Update, Delete), agents MUST update the AI Provenance metadata.**

AI Provenance is rendered in the content template as a transparency block showing readers how the content was created.

### 8.1 What AI Provenance Displays

The template renders an "AI Provenance" section showing:

| Field | Description | Example |
|-------|-------------|---------|
| Agent | Agent that created/modified content | `Twitter Agent @twitter` |
| Why This Was Created | Agent's summary of purpose | "Created thread: AI Productivity Paradox - debunking 10x claims" |
| Category | Content category | Contrarian Take, Data Deep-Dive |
| Platform | Target platform | twitter |
| Created | Original creation date | Jan 26, 2026 |

### 8.2 Provenance Fields in Frontmatter

```yaml
---
# ... other fields ...
agent: "twitter"
agent_summary: "Created thread: [Title] - [Brief description of why]"
category: "Contrarian Take"
platform: "twitter"
created_at: "2026-01-26"
updated_at: "2026-01-27"  # If updated
---
```

### 8.3 When to Update Provenance

| Operation | Action Required |
|-----------|-----------------|
| **Create** | Set all provenance fields. `agent_summary` explains WHY this content was created. |
| **Update** | Update `updated_at` and append to `agent_summary` explaining what changed. |
| **Delete** | No action (content removed). |

### 8.4 Agent Summary Guidelines

The `agent_summary` is displayed as "Why This Was Created". It should:
- Be concise (1-2 sentences)
- Explain the PURPOSE, not just the action
- Include the content title
- Reference the contrarian angle if relevant

**GOOD:** `"Created thread: AI Developer Replacement Reality - debunking productivity hype with production data."`
**BAD:** `"Created thread"` (too vague)

### 8.5 Update Provenance After Operations

```json
// MCP: systemprompt_cli - Set provenance fields
{ "command": "core content edit --slug [slug] --source twitter --set agent=twitter --set agent_summary=\"Created thread: [Title] - [Why]\" --set category=\"Contrarian Take\"" }
```

Verify provenance is set:

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug [slug] --source twitter" }
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
| Thread too short | Specify exact tweet count: "8 tweets" |
| "Keywords cannot be empty" error | Include keywords array in content_create message |
| Hashtags/emojis appearing | Report bug - strictly forbidden in skill |
| Wrong numbering (1/, 2/ instead of 1/8, 2/8) | Agent prompt specifies N/Total format - older content may have this issue |
| Context lost between messages | Always use `--context-id "$CONTEXT_ID"` for every message |
| Signature URL wrong | Fixed in MCP - URL now auto-corrected to match slug |

### Context-Related Failures

See the [Session Playbook](../cli/session.md#context-management-quick-reference) and [Contexts Playbook](../cli/contexts.md) for context contamination symptoms and solutions. The fix is always: **create a NEW context and start fresh.**

---
title: "LinkedIn Content Creation Playbook"
description: "Create professional thought leadership content that translates technical insights into business value for CTOs and technical leaders."
keywords:
  - linkedin
  - thought-leadership
  - business
  - cto
---

# LinkedIn Content Creation Playbook

Create professional thought leadership content that translates technical insights into business value for CTOs and technical leaders.

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

**WRONG:** Reusing `linkedin-general` context for multiple posts
**RIGHT:** `linkedin-ai-tco-post`, `linkedin-agent-adoption`, `linkedin-rag-costs` (one context per content piece)

---

## Execution Model: One Tool Per Message

**CRITICAL: Each step is a SEPARATE message. NEVER combine steps.**

1. **Trend Context** - Provide the trend and angle
2. **Goal Confirmation** - Agent confirms the approach
3. **Research** - Agent calls research_content, waits for response
4. **Create** - Agent calls content_create in next message
5. **Response** - Agent summarises result

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Plan | Provide trend context | YES - share trend details |
| Research | Call research_content | YES - send message, wait for response |
| Create | Call content_create | YES - send message, wait for response |
| Image | Generate image | YES - optional |

**RIGHT:** "Research X" → wait for response → "Now create the post using research_id ABC"
**WRONG:** Combining research and creation in one message or tool plan

---

## Step 1: Create a FRESH Context (REQUIRED FIRST STEP)

**Every new piece of content MUST start with a fresh context.** Name it descriptively for what you're creating.

```json
// MCP: systemprompt_cli
{ "command": "core contexts list" }
```

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"linkedin-[topic-slug]\"" }
```

**Examples of good context names:**
```json
{ "command": "core contexts new --name \"linkedin-ai-tco-reality\"" }
{ "command": "core contexts new --name \"linkedin-agent-adoption-costs\"" }
{ "command": "core contexts new --name \"linkedin-rag-hidden-expenses\"" }
```

**ONLY continue an existing context if:**
- You're resuming work on the SAME content piece
- You haven't created any other content since
- The context was created specifically for this content

```json
// MCP: systemprompt_cli - ONLY if continuing the SAME content piece
{ "command": "core contexts use linkedin-ai-tco-reality" }
```

## Step 2: Review Performance and Identify a Topic

### 2.1 Review Content Performance

```json
// MCP: systemprompt_cli
{ "command": "analytics content --days 30" }
```

```json
// MCP: systemprompt_cli
{ "command": "analytics platform --days 30" }
```

### 2.2 Identify a Topic

Choose a topic that:
1. Has a clear **business angle** (cost, risk, efficiency, competitive advantage)
2. CTOs/VPs would care about (affects budget, headcount, strategy)
3. Is timely or evergreen with a fresh perspective

### 2.3 Get Context ID

```json
// MCP: systemprompt_cli - First message, capture the context ID from response
{ "command": "admin agents message linkedin -m \"I found a trending topic with business implications. Let me share the details.\" --blocking --json" }
```

**Save the CONTEXT_ID for all subsequent messages.**

## Step 3: Provide Trend Context (NEVER FABRICATE)

**CRITICAL: NEVER INVENT OR FABRICATE DATA**

Do NOT provide:
- Made-up statistics or percentages
- Fabricated "production experience"
- Invented case studies or outcomes
- Fake metrics to sound credible

The agent will write opinion and analysis based on the trend. That's fine. What's not fine is pretending to have evidence you don't have.

### 3.1 Send Trend Information

```json
// MCP: systemprompt_cli
{ "command": "admin agents message linkedin -m \"I want to create content about:\n\nTOPIC: '[topic]'\nANGLE: [angle]\nTARGET AUDIENCE: [who would care - CTOs, VPs, senior engineers]\n\nPlease research and create a LinkedIn post on this topic.\" --context-id \"$CONTEXT_ID\" --blocking" }
```

### 3.2 Example (Honest, No Fabrication)

```json
// MCP: systemprompt_cli
{ "command": "admin agents message linkedin -m \"I found a trending topic:\n\nTREND: 'LLM Structured Outputs Handbook' (HN, 337 points, 59 comments)\nSOURCE: https://nanonets.com/cookbooks/structured-llm-outputs\n\nANGLE: The handbook covers constrained vs unconstrained decoding - this is actually important infrastructure that gets ignored in the AI hype cycle.\n\nTARGET AUDIENCE: Engineering leaders evaluating LLM integrations.\n\nPlease research this and write a LinkedIn post with your analysis.\" --context-id \"$CONTEXT_ID\" --blocking" }
```

**Note:** No fake statistics. No invented "production experience". Just the trend and an angle.

---

## Step 4: Research (Separate Message)

Once the agent confirms the angle, tell it to proceed with research.

```json
// MCP: systemprompt_cli
{ "command": "admin agents message linkedin -m \"Proceed with research.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

**The agent will:**
- Research the topic
- Return a `research_id` artifact
- Wait for next instruction

**Save the research_id from the response for Step 5.**

## Step 5: Create Content (Separate Message)

After research completes, tell the agent to create the content.

```json
// MCP: systemprompt_cli
{ "command": "admin agents message linkedin -m \"Create the post using research. Use these specifics:\n- slug: '[url-friendly-slug]'\n- keywords: ['keyword1', 'keyword2', 'keyword3']\n- content_style: 'analytical'\n\nWrite opinion and analysis, not fabricated evidence.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 180" }
```

**Verify content was created:**

```json
// MCP: systemprompt_cli
{ "command": "core content search \"[slug]\"" }
```

---

## Step 6: Quality Review (REQUIRED - NEVER SKIP)

After content_create returns, the agent MUST validate the content:

```
# CONTENT CREATED:
# - ID: [content_id]
# - Slug: [slug]
# - Words: [count] / 420-450 required
# - Chars: [count] / 2,400-2,850 required
#
# VALIDATION:
# ✓/✗ Length meets requirement
# ✓/✗ No hashtags
# ✓/✗ Signature present and correct
# ✓/✗ Matches original brief
```

**If validation fails:**
- Ask agent to regenerate with specific fixes
- Or manually edit and re-ingest

**Checklist before proceeding:**
- [ ] Word count 420-450 (character count 2,400-2,850)
- [ ] No hashtags anywhere
- [ ] No emojis
- [ ] No markdown asterisks (bold/italic)
- [ ] Signature present: "The code doesn't write itself. Yet."
- [ ] URL uses /linkedin/[slug] NOT /blog/
- [ ] First 140 chars work standalone (before "see more")
- [ ] Matches original topic and angle from planning

## Step 7: Generate Image (Optional)

```json
// MCP: systemprompt_cli
{ "command": "admin agents message linkedin -m \"Generate a professional image for this post.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 60" }
```

## Step 8: Publish and Verify

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
{ "command": "cloud sync local content --direction to-disk --source linkedin -y" }
```

```json
// MCP: systemprompt_cli - Verify the content is accessible and published
{ "command": "core content verify --slug ai-cost-reality-ctos" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug ai-cost-reality-ctos" }
```

---

## Content Types

| Type | Length (Characters) | Words | Use Case |
|------|---------------------|-------|----------|
| `linkedin_post` | 2,400-2,850 | 420-450 | Thought leadership (MINIMUM 2,400 chars) |

**IMPORTANT: We ONLY create linkedin_post. We do NOT create linkedin_article.**

**CRITICAL**: LinkedIn posts under 2,400 characters (370 words) are INCOMPLETE and should be regenerated.

## MCP Tools Used by Agent

| Tool | Purpose |
|------|---------|
| `research_content` | Research topic, returns `research_id` |
| `content_create` | Create content from research |
| `generate_image` | Generate professional imagery |
| `analytics_query` | Query performance data |

## Format Requirements

- First 130 characters are critical (before "see more")
- Short paragraphs with line breaks
- Use → for lists, NOT bullets
- British English

**Required signature:**
```
The code doesn't write itself. Yet.

https://tyingshoelaces.com/linkedin/[slug]
```

**Forbidden:** Hashtags, emojis, em-dashes, bold/italic markdown

---

## CRITICAL: Content URL Structure

**Each content type lives in its own section on the website. DO NOT confuse them.**

| Content Type | URL Pattern | Filesystem Location |
|--------------|-------------|---------------------|
| LinkedIn post | `/linkedin/[slug]` | `services/content/linkedin/[slug]/index.md` |
| Blog post | `/blog/[slug]` | `services/content/blog/[slug]/index.md` |
| Medium article | `/medium/[slug]` | `services/content/medium/[slug]/index.md` |
| Twitter thread | `/twitter/[slug]` | `services/content/twitter/[slug]/index.md` |

**LinkedIn posts are NOT blog posts.** They:
- Live at `https://tyingshoelaces.com/linkedin/[slug]`
- Are stored in `services/content/linkedin/`
- Have `kind: "linkedin"` in frontmatter
- Are standalone content, not derived from blog posts

**When creating a LinkedIn post:**
- The signature URL MUST use `/linkedin/` not `/blog/`
- The post appears in the LinkedIn section of the website
- It is separate content from any blog post

---

## Example Session (No Fabrication)

```json
// Step 1: Create context
{ "command": "core contexts new --name \"linkedin-structured-outputs\"" }

// Step 2: Get CONTEXT_ID
{ "command": "admin agents message linkedin -m \"I found a trending topic.\" --blocking --json" }
// Save CONTEXT_ID from response

// Step 3: Provide trend context (NO FABRICATED DATA)
{ "command": "admin agents message linkedin -m \"I found a trending topic:\n\nTREND: 'LLM Structured Outputs Handbook' (HN, 337 points, 59 comments)\nSOURCE: https://nanonets.com/cookbooks/structured-llm-outputs\n\nANGLE: This handbook covers constrained vs unconstrained decoding - important infrastructure ignored in AI hype.\nTARGET AUDIENCE: Engineering leaders evaluating LLM integrations.\n\nLet me know if this angle works.\" --context-id \"$CONTEXT_ID\" --blocking" }
// Agent confirms or refines the angle

// Step 4: Research (SEPARATE MESSAGE)
{ "command": "admin agents message linkedin -m \"Proceed with research.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
// Agent calls research_content, returns research_id

// Step 5: Create (SEPARATE MESSAGE using research_id)
{ "command": "admin agents message linkedin -m \"Create the post using research. Use these specifics:\n- slug: 'structured-outputs-matter'\n- keywords: ['LLM', 'structured outputs', 'constrained decoding']\n- content_style: 'analytical'\n\nWrite analysis and opinion based on the trend.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 180" }
// Agent calls content_create with research_id

// VERIFY content exists
{ "command": "core content search \"structured-outputs-matter\"" }

// Step 7: Publish
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run publish_content" }
```

---

## Step 9: Validate Filesystem and Refine Content (FINAL QUALITY GATE - DOUBLE EDIT)

After publishing, validate the content has been synced to the filesystem and perform a final quality review.

### 9.1 Validate Filesystem Sync

See the [Content Playbook](../cli/content.md) for content management commands.

```json
// MCP: systemprompt_cli
{ "command": "core content status --source linkedin" }
```

**Expected frontmatter fields:**
```yaml
---
title: "Post Title"
description: "Brief description"
author: "Edward"
slug: "ai-cost-reality-ctos"
keywords: "keyword1, keyword2"
image: "/files/images/..."
kind: "linkedin"
public: true
tags: []
published_at: "2026-01-17"
---
```

### 9.2 Review Against Original Aim

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug ai-cost-reality-ctos --source linkedin" }
```

**Evaluation checklist:**
- [ ] Hook captures the agreed business angle from planning
- [ ] Length meets requirement (2,400-2,850 characters / 370-450 words) - COUNT IT!
- [ ] First 130 characters work standalone (before "see more")
- [ ] No hashtags, emojis, or em-dashes
- [ ] No markdown formatting (bold/italic shows as asterisks on LinkedIn)
- [ ] Signature present: "The code doesn't write itself. Yet."
- [ ] URL matches slug: `https://tyingshoelaces.com/linkedin/[slug]`
- [ ] British English (realise, optimise)
- [ ] No AI anti-patterns (colon setups, "turns out", stacked questions)

### 9.3 Edit to World-Class Quality

If the content needs refinement, edit directly in the filesystem, then re-ingest:

```json
// MCP: systemprompt_cli
{ "command": "core content ingest services/content/linkedin --source linkedin --override" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug ai-cost-reality-ctos --source linkedin" }
```

**Refinement focus areas:**
- Strengthen the hook if it doesn't grab attention immediately
- Add specific numbers, data points, or business outcomes
- Remove any generic AI-sounding phrases
- Ensure the question drives genuine engagement (not generic "What do you think?")
- Tighten prose - every sentence must earn its place

### 9.4 Final Verification

```json
// MCP: systemprompt_cli
{ "command": "core content verify --slug ai-cost-reality-ctos --source linkedin --base-url https://tyingshoelaces.com" }
```

**Content is ONLY complete when:**
1. ✓ Exists in database (`core content show` succeeds)
2. ✓ Exists on filesystem (`services/content/linkedin/[slug]/index.md` exists)
3. ✓ Frontmatter is valid YAML with all required fields
4. ✓ Meets all skill requirements (length, format, voice)
5. ✓ Matches the original planning intent
6. ✓ Published and accessible on site

---

## Step 10: AI Provenance (Automatic)

AI Provenance is **automatically created** when content is generated via the MCP `content_create` tool.

### 10.1 What AI Provenance Displays

| Field | Description | Example |
|-------|-------------|---------|
| Agent | Agent that created content | `LinkedIn Agent @linkedin` |
| Why This Was Created | Agent's summary of purpose | "Created post: AI TCO Reality Check - challenging vendor claims" |
| Category | Content category | Thought Leadership, Industry Analysis |
| Platform | Target platform | linkedin |
| Created | Original creation date | Jan 26, 2026 |

### 10.2 How Provenance is Created

Provenance is stored in database tables (NOT frontmatter):
- **`content_agent_attribution`** - Agent identity and creation reason
- **`content_categorization`** - Category, platform, content style

The MCP `content_create` tool automatically populates these during content creation.

### 10.3 Verify Provenance

```json
// MCP: systemprompt_cli
{ "command": "infra db query \"SELECT agent_name, creation_reason FROM content_agent_attribution WHERE content_id = '[content-id]'\"" }
```

**Provenance is automatic. The MCP content creation tools handle it.**

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
| Content too long | Specify `linkedin_post` for shorter content |
| "Keywords cannot be empty" error | Include keywords array in content_create message |
| Hashtags appearing | Report bug - strictly forbidden in skill |
| Asterisks (*word*) in content | Fixed in MCP - markdown now stripped for LinkedIn |
| Context lost between messages | Always use `--context-id "$CONTEXT_ID"` for every message |
| Signature URL wrong | Fixed in MCP - URL now auto-corrected to match slug |

### Context-Related Failures

See the [Session Playbook](../cli/session.md#context-management-quick-reference) and [Contexts Playbook](../cli/contexts.md) for context contamination symptoms and solutions. The fix is always: **create a NEW context and start fresh.**

### Known MCP Issues (Jan 2026)

| Issue | Status | Workaround |
|-------|--------|------------|
| Task stuck in "working" after MCP tool completes | **OPEN** | Check if content was created despite timeout. Use `core content search` to verify. |
| research_content empty Gemini response | **OPEN** | Retry with more specific topic or different keywords |
| Content generated under length requirement | **MITIGATED** | Agent instructions updated to enforce validation. Check character count. |

See `plan/feedback/` for detailed issue documentation.

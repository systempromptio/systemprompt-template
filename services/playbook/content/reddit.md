---
title: "Reddit Content Creation Playbook"
description: "Create community-first engagement on technical subreddits with production war stories and data-backed contrarian takes. Value first, promotion never."
keywords:
  - reddit
  - community
  - technical
  - war-stories
---

# Reddit Content Creation Playbook

Create community-first engagement on technical subreddits with production war stories and data-backed contrarian takes. Value first, promotion never.

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

**WRONG:** Reusing `reddit-general` context for multiple posts
**RIGHT:** `reddit-ai-costs-ml`, `reddit-rag-localllama`, `reddit-career-expdevs` (one context per post)

---

## CRITICAL: One Step = One Message

**Each step in this playbook is a SEPARATE message to the agent. NEVER combine steps.**

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Step 3 | Plan | YES - send message, wait for response |
| Step 4 | Research | YES - send message, wait for response |
| Step 5 | Create | YES - send message, wait for response |

**WRONG:** "Research and create a post about X"
**RIGHT:** Message 1: "Research X" → wait → Message 2: "Create the post"

---

## Step 1: Create a FRESH Context (REQUIRED FIRST STEP)

**Every new post MUST start with a fresh context.** Name it descriptively including the target subreddit.

```json
// MCP: systemprompt_cli
{ "command": "core contexts list" }
```

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"reddit-[topic]-[subreddit]\"" }
```

**Examples of good context names:**
```json
{ "command": "core contexts new --name \"reddit-ai-costs-machinelearning\"" }
{ "command": "core contexts new --name \"reddit-llm-scaling-localllama\"" }
{ "command": "core contexts new --name \"reddit-career-growth-expdevs\"" }
```

**ONLY continue an existing context if:**
- You're resuming work on the SAME post
- You haven't created any other content since
- The context was created specifically for this post

```json
// MCP: systemprompt_cli - ONLY if continuing the SAME post
{ "command": "core contexts use reddit-ai-costs-machinelearning" }
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
{ "command": "admin agents message reddit -m \"Our top Reddit posts performed well in [subreddits]. What production experience should we share next?\" --blocking --json" }
```

**Save the CONTEXT_ID for all subsequent messages.**

## Step 3: SOCRATIC DIALOGUE - Convince the Gatekeeper (MANDATORY)

**CRITICAL: The Reddit agent is a GATEKEEPER. It will NOT research or create content until you have convinced it with deep, articulated reasoning about the COMMUNITY VALUE.**

The agent uses the Socratic method to interrogate your content idea. Your job is to provide clear, specific, well-reasoned answers that demonstrate you understand what the specific subreddit values.

### 3.1 Initiate the Dialogue

```json
// MCP: systemprompt_cli
{ "command": "admin agents message reddit -m \"I want to write a Reddit post about [topic] for [subreddit]. Let's define the community value before researching.\" --context-id \"$CONTEXT_ID\" --blocking" }
```

The agent will ask probing questions. **DO NOT skip this step or give vague answers.**

### 3.2 Be Prepared to Answer These Questions

**On Subreddit Fit:**
- Which subreddit is this for? r/MachineLearning, r/LocalLLaMA, r/programming, or r/ExperiencedDevs?
- What does this community specifically care about?
- What would get this downvoted in that subreddit?
- What's the culture - academic rigor? practical code? cost obsession?

**On Value & Authenticity:**
- What's the standalone value if nobody clicks your link?
- What personal experience backs this up?
- Why would a cynical Redditor upvote this?
- What are you sharing that they can't Google in 30 seconds?

**On Evidence:**
- What specific numbers, benchmarks, or production data do you have?
- Is this from personal experience, benchmarks, code examples, or cost data?
- What failure or mistake taught you this?

**On Depth:**
- Is this a quick insight (300 words), detailed analysis (800 words), or deep dive (1500+ words)?
- Does the value justify the length?

### 3.3 Example Dialogue

**WRONG (Vague - Agent Will Reject):**
```json
// MCP: systemprompt_cli
{ "command": "admin agents message reddit -m \"Write about AI costs\" --context-id \"$CONTEXT_ID\" --blocking" }
// Agent will push back: "Which subreddit? What personal experience? What specific data?"
```

**RIGHT (Specific - Agent Will Accept):**
```json
// MCP: systemprompt_cli
{ "command": "admin agents message reddit -m \"I want to write a Reddit post for r/MachineLearning about the hidden costs of production AI.\n\nTARGET SUBREDDIT: r/MachineLearning - they value rigorous analysis and production data over hype.\n\nCOMMUNITY VALUE: Senior ML engineers budgeting AI projects need real cost breakdowns, not vendor pricing pages. This gives them data to push back on unrealistic budget expectations.\n\nPERSONAL EXPERIENCE: We ran 3 production LLM systems for 18 months. Inference costs were 20% of actual spend. The rest was data engineering, monitoring, human review, and incident response.\n\nSPECIFIC DATA:\n- Inference: $2,400/month average\n- Data pipeline maintenance: $4,200/month in engineering time\n- Quality monitoring: $3,100/month\n- Human review for hallucinations: $5,800/month\nTotal: $15,500/month vs $2,400 that 'AI costs' conversations focus on.\n\nPOST DEPTH: Detailed analysis (800-1000 words) with cost breakdown table.\n\nHOOK: 'Our AI budget was 6x what we planned. Not because of GPT-4 pricing.'\n\nLet's refine this before researching.\" --context-id \"$CONTEXT_ID\" --blocking" }
```

### 3.4 Gate Check - Agent Must Confirm

The agent will NOT proceed to research until it can articulate:
- The target subreddit and why it fits
- The standalone value being shared
- The evidence type backing claims
- The appropriate post depth

**Wait for the agent to confirm the goal:**
```
CONFIRMED REDDIT CONTENT GOAL:
- Subreddit: [target subreddit]
- Value proposition: [what the community gets]
- Evidence type: [personal_experience|benchmarks|code_examples|cost_data|failure_analysis]
- Content category: [production_war_story|cost_breakdown|architecture_critique|tool_evaluation|contrarian_analysis]
- Post depth: [quick_insight|detailed_analysis|deep_dive]
- Hook style: [how we open to earn attention]
```

**Only then will the agent proceed to research.**

---

## Step 4: Research the Topic (Agent-Initiated After Goal Confirmed)

Once the agent confirms the goal, ask it to research:

```json
// MCP: systemprompt_cli
{ "command": "admin agents message reddit -m \"The goal is confirmed. Research this topic using research_content with content_type='reddit_post'. Return the research_id when done.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

**WAIT for response. Save the research_id (e.g., `RESEARCH_ID="..."`). Then proceed to Step 5.**

## Step 5: Create the Content (With Precise Guidance From Dialogue)

The agent will include everything from the Socratic dialogue in the `instructions` field automatically. You can add specifics:

```json
// MCP: systemprompt_cli
{ "command": "admin agents message reddit -m \"Create the Reddit post using content_create with:\n- research_id='$RESEARCH_ID'\n- slug='production-ai-hidden-costs'\n- content_type='reddit_post'\n- keywords=['AI costs', 'production ML', 'LLM', 'budget', 'machine learning']\n\nAdditional instructions: Include the full cost breakdown table we discussed. Open with the 6x budget overrun hook. Target r/MachineLearning tone (rigorous, data-first). Add 'Happy to share more details if useful' closing.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
```

**Critical:** The agent will embed the confirmed subreddit, value proposition, evidence, and hook from the Socratic dialogue into the content creation.

**WAIT for response. Verify content exists:**

```json
// MCP: systemprompt_cli
{ "command": "core content search \"production-ai-hidden-costs\"" }
```

**If the agent says content was created but it doesn't exist, try again with explicit parameters.**

## Step 6: Publish and Verify

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
{ "command": "cloud sync local content --direction to-disk --source reddit -y" }
```

```json
// MCP: systemprompt_cli - Verify the content is accessible and published
{ "command": "core content verify --slug production-ai-hidden-costs" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug production-ai-hidden-costs" }
```

---

## Content Types

| Type | Length | Use Case |
|------|--------|----------|
| `reddit_post` | 300-2000 words | All subreddits (depth varies by topic) |

### Post Depth Guidelines

| Depth | Words | When to Use |
|-------|-------|-------------|
| Quick insight | 300-500 | Single data point, one lesson learned |
| Detailed analysis | 600-1000 | Cost breakdown, tool comparison, architecture decision |
| Deep dive | 1000-2000 | Production postmortem, comprehensive guide, career journey |

## Target Subreddits

| Subreddit | Culture | What Works | What Fails |
|-----------|---------|------------|------------|
| r/MachineLearning | Academic rigor | Papers, production data, benchmarks | Hype, promotional, vague claims |
| r/programming | Practical over theory | Code examples, real problems, war stories | Theory without code, promotional |
| r/LocalLLaMA | Cost obsessed, anti-cloud | Self-hosting, cost comparisons, local solutions | Cloud advocacy, expensive setups |
| r/ExperiencedDevs | Career focus, production reality | Career lessons, senior perspectives, honest takes | Beginner questions, naive advice |

## MCP Tools Used by Agent

| Tool | Purpose |
|------|---------|
| `research_content` | Topic research, returns `research_id` |
| `content_create` | Create content from research |
| `generate_image` | Generate AI images |

## Format Requirements

**Structure:**
- tl;dr at the very top (for posts over 500 words)
- Markdown headers (##) for sections
- Code blocks with language tags
- Tables for data comparisons

**Closing style:**
```
Happy to answer questions. Full writeup with code if there's interest.

[Link to blog if relevant]
```

**Forbidden:** Self-promotion in title, hashtags, emojis, marketing speak

---

## Example Session (Each Step is Separate)

```json
// Step 1: Create context
{ "command": "core contexts new --name \"reddit-ai-costs-ml\"" }

// Step 2: Review trends and get CONTEXT_ID
{ "command": "analytics content top --since 30d --limit 5" }

// First message - capture the context ID from response
{ "command": "admin agents message reddit -m \"Our production AI posts do well on r/MachineLearning. What cost data would be most valuable to share?\" --blocking --json" }
// Save CONTEXT_ID from response

// Step 3: Plan (SEND, WAIT FOR RESPONSE)
{ "command": "admin agents message reddit -m \"I want to write about production AI costs for r/MachineLearning. We have 18 months of real cost data across 3 systems. What angle would provide most value?\" --context-id \"$CONTEXT_ID\" --blocking" }

// Step 4: Research (SEPARATE MESSAGE, WAIT FOR RESPONSE)
{ "command": "admin agents message reddit -m \"Research AI production costs, TCO analysis, and cost optimization strategies. Return the research_id when done.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }
// Save the research_id from response, e.g.: RESEARCH_ID="4a7f8ac2-..."

// Step 5: Create (SEPARATE MESSAGE, WAIT FOR RESPONSE)
{ "command": "admin agents message reddit -m \"Create a Reddit post with:\n- research_id='$RESEARCH_ID'\n- slug='production-ai-real-costs'\n- content_type='reddit_post'\n- keywords=['AI costs', 'production ML', 'TCO', 'LLM', 'budget']\n\nDetailed analysis depth (800 words). Include our cost breakdown table. Use the hook we discussed.\" --context-id \"$CONTEXT_ID\" --blocking --timeout 120" }

// VERIFY content exists - CRITICAL STEP
{ "command": "core content search \"production-ai-real-costs\"" }

// Step 6: Optimize images, Publish, and Sync to filesystem
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run publish_content" }
{ "command": "cloud sync local content --direction to-disk --source reddit -y" }
{ "command": "core content verify --slug production-ai-real-costs" }
```

---

## Step 7: Validate Filesystem and Refine Content (FINAL QUALITY GATE)

After publishing, validate the content has been synced to the filesystem and perform a final quality review.

### 7.1 Validate Filesystem Sync

See the [Content Playbook](../cli/content.md) for content management commands.

**IMPORTANT:** Content created by agents lives in the database. You MUST sync to filesystem before validating:

```json
// MCP: systemprompt_cli - Sync database content to filesystem
{ "command": "cloud sync local content --direction to-disk --source reddit -y" }
```

```json
// MCP: systemprompt_cli - Verify sync status
{ "command": "core content status --source reddit" }
```

**Expected frontmatter fields:**
```yaml
---
title: "Post Title"
description: "Brief description"
author: "Edward"
slug: "production-ai-real-costs"
keywords: "keyword1, keyword2"
image: "/files/images/..."
kind: "reddit"
public: true
tags: []
published_at: "2026-01-17"
---
```

### 7.2 Review Against Original Aim

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug production-ai-real-costs --source reddit" }
```

**Evaluation checklist:**
- [ ] tl;dr present at top (if over 500 words)
- [ ] Matches target subreddit culture (r/MachineLearning = rigorous, data-first)
- [ ] Contains specific numbers/data (not vague claims)
- [ ] Personal experience anchors the post
- [ ] No promotional language or self-promotion in title
- [ ] No hashtags or emojis
- [ ] Markdown headers structure the content
- [ ] Code blocks have language tags (if code present)
- [ ] Closing invites discussion without being needy
- [ ] British English (realise, optimise)

### 7.3 Edit to World-Class Quality

If the content needs refinement, edit directly in the filesystem, then re-ingest:

```json
// MCP: systemprompt_cli
{ "command": "core content ingest services/content/reddit --source reddit --override" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug production-ai-real-costs --source reddit" }
```

**Refinement focus areas:**
- Strengthen the tl;dr - it must capture the key insight
- Ensure data is specific and credible (exact numbers, not "roughly" or "about")
- Check subreddit tone match (r/LocalLLaMA wants cost obsession, r/ExperiencedDevs wants career wisdom)
- Remove any hint of promotion or marketing speak
- Add specific details: dates, versions, tools used
- Verify the closing feels like a community member, not a marketer

### 7.4 Final Verification

```json
// MCP: systemprompt_cli
{ "command": "core content verify --slug production-ai-real-costs --source reddit --base-url https://tyingshoelaces.com" }
```

**Content is ONLY complete when:**
1. Exists in database (`core content show` succeeds)
2. Exists on filesystem (`services/content/reddit/[slug]/index.md` exists)
3. Frontmatter is valid YAML with all required fields
4. Matches target subreddit culture
5. Contains specific, credible evidence
6. No promotional elements
7. Published and accessible on site

---

## Step 8: AI Provenance (Automatic)

AI Provenance is **automatically created** when content is generated via the MCP `content_create` tool.

### 8.1 What AI Provenance Displays

| Field | Description | Example |
|-------|-------------|---------|
| Agent | Agent that created content | `Reddit Agent @reddit` |
| Why This Was Created | Agent's summary of purpose | "Created post: Production AI Hidden Costs - sharing real cost data" |
| Category | Content category | Production War Story, Cost Breakdown |
| Platform | Target platform | reddit |
| Created | Original creation date | Jan 26, 2026 |

### 8.2 How Provenance is Created

Provenance is stored in database tables (NOT frontmatter):
- **`content_agent_attribution`** - Agent identity and creation reason
- **`content_categorization`** - Category, platform, content style

The MCP `content_create` tool automatically populates these during content creation.

### 8.3 Verify Provenance

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
| Post feels promotional | Rewrite to lead with value. Remove all links except one at the end. |
| "Keywords cannot be empty" error | Include keywords array in content_create message |
| Context lost between messages | Always use `--context-id "$CONTEXT_ID"` for every message |
| Wrong subreddit tone | Be explicit: "Match r/LocalLLaMA tone: cost-obsessed, anti-cloud" |

### Context-Related Failures

See the [Session Playbook](../cli/session.md#context-management-quick-reference) and [Contexts Playbook](../cli/contexts.md) for context contamination symptoms and solutions. The fix is always: **create a NEW context and start fresh.**

---

## Community-First Principles

Reddit content must **provide value first**. The community can smell promotion instantly.

**DO:**
- Lead with the insight, not the product
- Share real numbers from real experience
- Admit failures and what you learned
- Respond to comments genuinely
- Ask the community for their experience

**DON'T:**
- Put your site/product in the title
- Write like a blog post (tl;dr comes FIRST on Reddit)
- Use marketing language ("revolutionary", "game-changing")
- Post and disappear
- Cross-post the same content everywhere

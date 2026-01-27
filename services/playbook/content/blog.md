---
title: "Blog Content Creation Playbook"
description: "Create long-form technical blog content for tyingshoelaces.com. Narrative-driven, deeply personal, and technically precise."
keywords:
  - blog
  - content
  - writing
  - technical
  - longform
---

# Blog Content Creation Playbook

Create long-form technical blog content for tyingshoelaces.com. Narrative-driven, deeply personal, and technically precise.

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

**WRONG:** Reusing `blog-general` context for multiple posts
**RIGHT:** `blog-mcp-deep-dive`, `blog-ai-production-gap`, `blog-rag-architecture` (one context per post)

---

## Execution Model: One Tool Per Message

**CRITICAL: Each step is a SEPARATE message. NEVER combine steps.**

1. **Socratic Dialogue** (no tools) - Agent interrogates your content goal
2. **Goal Confirmation** - Agent confirms the plan
3. **Research** - Agent calls research_content, waits for response
4. **Create** - Agent calls content_create in next message
5. **Response** - Agent summarises result

| Step | Action | Separate Message? |
|------|--------|-------------------|
| Plan | Socratic Dialogue | YES - refine goals with agent |
| Research | Call research_content | YES - send message, wait for response |
| Create | Call content_create | YES - send message, wait for response |
| Image | Generate featured image | YES - send message, wait for response |

**RIGHT:** "Research X" -> wait for response -> "Now create the post using research_id ABC"
**WRONG:** Combining research and creation in one message or tool plan

---

## Step 1: Create a FRESH Context (REQUIRED FIRST STEP)

**Every new blog post MUST start with a fresh context.** Name it descriptively for what you're creating.

```json
// MCP: systemprompt_cli
{ "command": "core contexts list" }
```

```json
// MCP: systemprompt_cli
{ "command": "core contexts new --name \"blog-[topic-slug]\"" }
```

**Examples of good context names:**
```json
{ "command": "core contexts new --name \"blog-mcp-deep-dive\"" }
{ "command": "core contexts new --name \"blog-ai-production-gap\"" }
{ "command": "core contexts new --name \"blog-rag-architecture-lessons\"" }
```

**ONLY continue an existing context if:**
- You're resuming work on the SAME blog post
- You haven't created any other content since
- The context was created specifically for this post

```json
// MCP: systemprompt_cli - ONLY if continuing the SAME blog post
{ "command": "core contexts use blog-mcp-deep-dive" }
```

Once a context is active, all `agents message` commands automatically use it.

**IMPORTANT:** Due to a known issue, context may not persist automatically between messages. Always pass `--context-id` explicitly.

## Step 2: Review Performance and Choose a Topic

Before creating new content, review what's performing well.

### 2.1 Review Content Performance

```json
// MCP: systemprompt_cli
{ "command": "analytics content --days 30" }
```

```json
// MCP: systemprompt_cli
{ "command": "analytics category" }
```

### 2.2 Choose a Topic

Select a topic that:
1. Aligns with your expertise (AI, production systems, Rust, DevOps)
2. Is timely or provides a unique perspective
3. You have a contrarian or unique angle on

---

## Step 3: SOCRATIC DIALOGUE - Convince the Gatekeeper (MANDATORY)

**CRITICAL: The blog agent is a GATEKEEPER. It will NOT research or create content until you have convinced it with deep, articulated reasoning about the content goal.**

The agent uses the Socratic method to interrogate your content idea. Your job is to provide clear, specific, well-reasoned answers that demonstrate you've thought deeply about the value this content will provide.

### 3.1 Initiate the Dialogue

```json
// MCP: systemprompt_cli
{ "command": "admin agents message blog -m \"I want to write a blog post about [topic]. Let's define the goal before researching.\" --blocking" }
```

The agent will ask probing questions. **DO NOT skip this step or give vague answers.**

### 3.2 Be Prepared to Answer These Questions

**On Thesis:**
- What specific claim are you making that others would disagree with?
- If you summarised this in one sentence, what would it be?
- What's the 'so what' - why should anyone care?

**On Evidence:**
- What production experience do you have that backs this?
- What data, metrics, or concrete examples support this?
- What failures taught you this lesson?

**On Contrarian Angle:**
- What does the conventional wisdom say, and why is it wrong?
- Who will this post make uncomfortable, and why?
- What's the uncomfortable truth you're revealing?

**On Reader Value:**
- What will a senior engineer do differently after reading this?
- What decision will a CTO reconsider?
- What misconception are you correcting?

### 3.3 Example Dialogue

**WRONG (Vague - Agent Will Reject):**
```json
// MCP: systemprompt_cli
{ "command": "admin agents message blog -m \"Write about AI agents\" --blocking" }
// Agent will push back: "What specific claim? What evidence? What's the contrarian angle?"
```

**RIGHT (Trend-Informed + Specific - Agent Will Accept):**
```json
// MCP: systemprompt_cli
{ "command": "admin agents message blog -m \"I found a trending topic I want to write about:\n\nTREND: 'Microsoft pauses Claude Code rollout' is trending on Reddit (84 upvotes, 44 comments)\nSOURCE: https://reddit.com/r/ClaudeAI/comments/...\n\nTHESIS: The Claude Code vs Copilot battle reveals a fundamental truth - terminal-native AI tools beat IDE plugins because they work WITH developer workflows, not against them.\n\nCONTRARIAN ANGLE: Everyone thinks AI coding assistants are about autocomplete. Wrong. The real battle is agentic (Claude Code) vs autocomplete (Copilot). Microsoft pausing Claude Code isn't about competition - it's about realising their paradigm is losing.\n\nEVIDENCE: I've used both extensively in production. Claude Code's agentic approach reduced my context-switching by 70%. Copilot's autocomplete interrupts flow. The terminal is where real work happens.\n\nREADER OUTCOME: Senior engineers will reconsider their IDE-centric workflow. CTOs will understand why terminal-native tools are winning. Teams will evaluate AI tools based on workflow integration, not feature lists.\n\nLet's refine this before researching.\" --blocking" }
```

### 3.4 Gate Check - Agent Must Confirm

The agent will NOT proceed to research until it can articulate:
- The thesis in one sentence
- The contrarian angle
- The production evidence
- The reader takeaway
- The hook direction

**Wait for the agent to confirm the goal:**
```
CONFIRMED CONTENT GOAL:
- Thesis: [one sentence]
- Contrarian angle: [what conventional wisdom we're challenging]
- Evidence base: [production experience/data we'll draw from]
- Reader outcome: [what they'll think/do differently]
- Hook direction: [opening that grabs]
```

**Only then will the agent proceed to research.**

---

## Step 4: Research (Separate Message)

Once the agent confirms the goal, tell it to proceed with research. The agent will call `research_content` and return a `research_id`.

```json
// MCP: systemprompt_cli
{ "command": "admin agents message blog -m \"Proceed with research.\" --blocking --timeout 120" }
```

**The agent will:**
- Research the topic (minimum 5 grounding reference links)
- Return a `research_id` artifact
- Wait for next instruction

**Save the research_id from the response for Step 5.**

## Step 5: Create Content (Separate Message)

After research completes, tell the agent to create the content using the research_id.

```json
// MCP: systemprompt_cli
{ "command": "admin agents message blog -m \"Create the blog post using research. Use these specifics:\n- slug: 'agent-mesh-architecture-production'\n- keywords: ['AI agents', 'agent mesh', 'production AI', 'microservices', 'orchestration']\n- Include the 3 specific failure cases we discussed\n- Add code examples showing the God Agent vs Mesh Agent patterns\n- Reference the 90% error reduction metric\n- British English, 3500+ words minimum\" --blocking --timeout 300" }
```

**The agent will:**
- Create 3500-5000 words of technical content
- Use British English (realise, optimise, colour)
- Include clear structure with scannable headers
- Back claims with production evidence
- Include code examples and technical details
- Honour the thesis and angle agreed in the dialogue

## Step 6: Generate Featured Image

Ask the agent to generate a featured image for the post.

```json
// MCP: systemprompt_cli
{ "command": "admin agents message blog -m \"Generate a featured image for this blog post. It should represent [the core technical concept] visually.\" --blocking --timeout 60" }
```

## Step 7: Publish and Verify

Publish the content to make it live on the site.

**IMPORTANT:** Image optimization MUST run before publish. The blog detail template expects `.webp` images, but content is created with `.png`. Without optimization, featured images will 404.

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
{ "command": "cloud sync local content --direction to-disk --source blog -y" }
```

```json
// MCP: systemprompt_cli - Verify the content is accessible and published
{ "command": "core content verify --slug mcp-architecture-deep-dive" }
```

```json
// MCP: systemprompt_cli - Show the content details
{ "command": "core content show --slug mcp-architecture-deep-dive" }
```

## MCP Tools Used by Agent

The Blog agent has access to these MCP tools (you don't call these directly):

| Tool | Purpose |
|------|---------|
| `research_content` | Research topic using web search, returns `research_id` |
| `content_create` | Create content from research, saves to database |
| `generate_image` | Generate AI image for featured image |
| `analytics_query` | Query performance data |
| `content_store` | Store ideas and suggestions |

## Content Requirements

Blog posts have strict requirements:

| Requirement | Value |
|-------------|-------|
| Length | 2000-4000 words |
| Grounding links | Minimum 5 reference links |
| Keywords | Required for SEO |
| Featured image | Required (placeholder used if not generated) |
| Language | British English |

## Voice and Style

- Lead with a compelling hook that challenges conventional wisdom
- Use personal experience and production data to back claims
- Technical depth with accessible explanations
- Contrarian takes that provoke thought
- Clear structure with scannable headers
- Sardonic humour about industry hype

## Example Session

```json
// Step 1: Create context
{ "command": "core contexts new --name \"blog-claude-code-vs-copilot\"" }

// Step 2: Socratic dialogue with topic
{ "command": "admin agents message blog -m \"I want to write about:\n\nTOPIC: 'Terminal-native AI tools vs IDE plugins'\nTHESIS: Terminal-native AI tools beat IDE plugins because they work WITH developer workflows.\nCONTRARIAN ANGLE: The real battle is agentic vs autocomplete, not Claude vs Copilot.\nEVIDENCE: I've used both - Claude Code reduced context-switching 70%.\nREADER OUTCOME: Engineers reconsider IDE-centric workflows.\n\nLet's refine before creating.\" --blocking" }
// Agent responds with questions or confirms goal

// Step 4: Research (after goal confirmed - SEPARATE MESSAGE)
{ "command": "admin agents message blog -m \"Proceed with research.\" --blocking --timeout 120" }
// Agent calls research_content, returns research_id

// Step 5: Create (SEPARATE MESSAGE using research_id)
{ "command": "admin agents message blog -m \"Create the blog post using research. Use these specifics:\n- slug: 'terminal-native-ai-tools-winning'\n- keywords: ['Claude Code', 'Copilot', 'AI coding', 'terminal', 'agentic AI']\n- Include code examples comparing workflows\n- Reference the Reddit trend\n- British English, 3500+ words minimum\" --blocking --timeout 300" }
// Agent calls content_create with research_id

// Step 6: Image
{ "command": "admin agents message blog -m \"Generate a featured image showing terminal vs IDE, simple vs complex\" --blocking --timeout 60" }

// Step 7: Optimize images, Publish, and Sync to filesystem
{ "command": "infra jobs run blog_image_optimization" }
{ "command": "infra jobs run publish_content" }
{ "command": "cloud sync local content --direction to-disk --source blog -y" }
{ "command": "core content show --slug terminal-native-ai-tools-winning" }
```

## Blog Post Structure

Typical structure for technical blog posts:

```markdown
# [Title - Hook that challenges]

[Opening paragraph - personal stake, why this matters]

## The Conventional Wisdom

[What everyone believes/says]

## What Actually Happens

[The reality from production experience]

## Why This Matters

[Technical deep-dive with code examples]

## The Data

[Production metrics, statistics, evidence]

## What To Do About It

[Actionable insights]

## Conclusion

[Provocative closing thought]
```

## Step 8: Validate Filesystem and Refine Content (FINAL QUALITY GATE)

After publishing, validate the content has been synced to the filesystem and perform a final quality review.

### 8.1 Validate Filesystem Sync

**IMPORTANT:** Content created by agents lives in the database. You MUST sync to filesystem before validating:

```json
// MCP: systemprompt_cli - Sync database content to filesystem
{ "command": "cloud sync local content --direction to-disk --source blog -y" }
```

```json
// MCP: systemprompt_cli - Verify sync status
{ "command": "core content status --source blog" }
```

**Expected frontmatter fields:**
```yaml
---
title: "Blog Post Title"
description: "Brief description"
author: "Edward"
slug: "ai-demo-production-gap"
keywords: "keyword1, keyword2, keyword3, keyword4"
image: "/files/images/blog/..."
kind: "blog"
public: true
tags: []
published_at: "2026-01-17"
---
```

### 8.2 Review Against Original Aim

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug ai-demo-production-gap --source blog" }
```

**Evaluation checklist:**
- [ ] Title is max 8 words, no colons or em-dashes
- [ ] Title is personal and specific (not generic "Best Practices" style)
- [ ] Length is 3,500-5,000 words
- [ ] Minimum 5 inline citations with full URLs and descriptive anchor text
- [ ] No domain-only citations like `[medium.com]` or `[reddit.com]`
- [ ] British English throughout (realise, optimise, colour)
- [ ] Structure follows: Prelude -> Problem -> Journey -> Lesson -> Conclusion
- [ ] 2-3 code examples included (if technical topic)
- [ ] No AI anti-patterns ("I discovered that...", "Fascinatingly...")
- [ ] No fake engagement questions
- [ ] Contrarian hook that challenges conventional wisdom

**CRITICAL: No Fabrication Check**
- [ ] All "I built", "we implemented", "my team" claims are backed by cited sources
- [ ] Metrics and statistics have citations (not invented)
- [ ] Personal anecdotes reference research, not fabricated experiences
- [ ] Code examples are from research sources or generic illustrations (not fake "production" code)

**If fabrication is detected:** Content must be regenerated with proper research citations. The agent MUST NOT invent experiences.

### 8.3 Edit to World-Class Quality

If the content needs refinement, edit directly in the filesystem, then re-ingest:

```json
// MCP: systemprompt_cli
{ "command": "core content ingest services/content/blog --source blog --override" }
```

```json
// MCP: systemprompt_cli
{ "command": "core content show --slug ai-demo-production-gap --source blog" }
```

**Refinement focus areas:**
- Strengthen the prelude - must hook within first 2-3 sentences
- Ensure personal story feels authentic, with specific details (dates, numbers, names)
- Check each section earns its place - remove any padding
- Verify code examples are real, not contrived
- Ensure citations use descriptive anchor text with full URLs
- Remove any hollow conclusions or generic takeaways
- Tighten prose - every paragraph must advance the narrative
- Check the lesson connects to bigger themes beyond the immediate topic
- Ensure the return to the opening in the conclusion lands

### 8.4 Final Verification

```json
// MCP: systemprompt_cli
{ "command": "core content verify --slug ai-demo-production-gap --source blog --base-url https://tyingshoelaces.com" }
```

**Content is ONLY complete when:**
1. Exists in database (`core content show` succeeds)
2. Exists on filesystem (`services/content/blog/[slug]/index.md` exists)
3. Frontmatter is valid YAML with all required fields
4. Word count is 3,500-5,000 words
5. Contains minimum 5 inline citations with full URLs
6. Title follows rules (max 8 words, no colons/em-dashes, personal)
7. Matches the contrarian hook and narrative arc agreed in planning
8. **NO FABRICATION** - All experiences/metrics backed by research citations
9. Published and accessible on site

---

## Step 9: AI Provenance (Automatic)

AI Provenance is **automatically created** when content is generated via the MCP `content_create` tool. It records who created the content and why.

### 9.1 What AI Provenance Displays

The template renders an "AI Provenance" section on each blog post showing:

```
┌─────────────────────────────────────────────────────────────────┐
│  AI Provenance                                                   │
├─────────────────────────────────────────────────────────────────┤
│  Blog Agent                                                      │
│  @blog                                                           │
│                                                                  │
│  Why This Was Created                                            │
│  "[Agent's summary of why this content was created]"             │
│                                                                  │
│  Content Strategy                                                │
│  Category        Platform                                        │
│  Personal Story  blog                                            │
│                                                                  │
│  Creation Context                                                │
│  Platform        Created                                         │
│  blog            Jan 26, 2026                                    │
│                                                                  │
│  This content was generated by an AI agent as part of an open    │
│  experiment in agentic content creation. We believe in full      │
│  transparency about how content is created and why.              │
│                                                                  │
│  Learn about our AI transparency practices                       │
└─────────────────────────────────────────────────────────────────┘
```

### 9.2 How Provenance is Created

Provenance is stored in two database tables (NOT in frontmatter):

**`content_agent_attribution`** - Agent identity and reason:
- `agent_name` - Agent that created content (e.g., "blog")
- `agent_display_name` - Display name (e.g., "Blog Agent")
- `creation_reason` - "Why This Was Created" (set by agent during creation)
- `creation_intent` - JSON with platform, skill_id, content_style, content_category

**`content_categorization`** - Content classification:
- `primary_category` - Category like "personal_story", "technical_deep_dive"
- `platform` - Target platform
- `content_style` - Style like "storytelling", "authoritative"

### 9.3 When Provenance is Set

| Operation | What Happens |
|-----------|--------------|
| **Create** | `content_create` MCP tool automatically records agent identity and creation reason |
| **Update** | New attribution record added with action_type="updated" |
| **Delete** | No provenance action |

### 9.4 Good Creation Reasons

The `creation_reason` (displayed as "Why This Was Created") is set by the agent during `content_create`. It should:
- Be concise (1-2 sentences)
- Explain the PURPOSE
- Include the content title

**GOOD:** `"Created blog post: ClawdBot - The AI That Acts, But Will Its Chat Break?"`
**BAD:** `"Created content"` (too vague)

### 9.5 Verify Provenance

Check provenance data via database query:

```json
// MCP: systemprompt_cli
{ "command": "infra db query \"SELECT agent_name, creation_reason, created_at FROM content_agent_attribution WHERE content_id = '[content-id]'\"" }
```

### 9.6 Transparency Statement

The template automatically appends:

> "This content was generated by an AI agent as part of an open experiment in agentic content creation. We believe in full transparency about how content is created and why."

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
| "Minimum 5 grounding links" error | Research phase incomplete; ask agent to research more |
| "Keywords cannot be empty" error | Include keywords array in content_create message |
| "Expected presentation_card artifact" error | **Create a NEW context**. Research artifact type mismatch from old session. |
| Content not created with requested slug | **Create a NEW context**. Agent reused prior context artifacts. |
| Content not publishing | Run `{ "command": "infra jobs run publish_content" }` |
| Featured image 404 on blog page | Run `{ "command": "infra jobs run blog_image_optimization" }` BEFORE `publish_content`. Blog template expects `.webp` but images are created as `.png`. |
| Image works on homepage but 404 on blog detail | Same as above - homepage uses `.png`, blog detail expects `.webp`. Run image optimization. |
| Missing featured image | Generate with `generate_image` or use default placeholder |
| Content too short | Explicitly state "MUST be 3000+ words minimum" |
| Agent auto-researches instead of planning | Add "Do NOT call research_content yet" to message |
| American English instead of British | Remind agent: "Use British English (realise, optimise)" |
| No code examples included | Explicitly request: "Include 2-3 code examples showing..." |

### Context-Related Failures

See the [Session Playbook](../cli/session.md) and [Contexts Playbook](../cli/contexts.md) for context contamination symptoms and solutions. The fix is always: **create a NEW context and start fresh.**
